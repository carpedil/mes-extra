use std::{collections::HashMap, env};

use common::{
    constants::DATABASE_URL,
    excel_helper::XlsxHelper,
    input::{ColumnDataInput, ExportSpecInput, SyncInput},
    output::{
        AppErr, AppResult, ColumnData, SyncedTableColumnsInfo, TableColumnsInfo, TableRawData,
    },
    utils::{gen_uid, get_user_tab_columns_sql},
};
use entity::{
    sync_table_columns_info::{self, Entity as SyncTableColumnsInfo},
    sync_tables::{self},
};
use regex::Regex;
use sea_orm::{
    prelude::Expr, ColumnTrait, DatabaseConnection, DbConn, DbErr, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};

use crate::{ConnectionConfigCmd, DatasourceCmd};

pub struct SyncTableCmd;
type MaxSyncNo = String;
type MaxSyncVersion = i32;

impl SyncTableCmd {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_db_conn() -> DbConn {
        let db = sea_orm::Database::connect(DATABASE_URL)
            .await
            .expect("can not connect to database, how old are you!");
        db
    }

    pub async fn sync_table_infos() -> Result<String, DbErr> {
        let mut table_columns: HashMap<(String, String), Vec<ColumnData>> = HashMap::new();
        if let Some(cc) = ConnectionConfigCmd::get_actived_config().await {
            let dq = DatasourceCmd::new(cc.clone());
            let data = dq
                .conn
                .query(&get_user_tab_columns_sql(cc.abandoned_table_list), &[])
                .unwrap();
            let mut row_list: Vec<ColumnData> = vec![];
            for row in data.into_iter() {
                let row = row.unwrap();
                let table_name: String = row.get("TABLE_NAME").unwrap();
                let table_desc: String = row.get("TAB_DESC").unwrap_or("".to_string());
                let column_name: String = row.get("COLUMN_NAME").unwrap();
                let column_desc: String = row.get("COL_DESC").unwrap_or("".to_string());
                let data_type: String = row.get("DATA_TYPE").unwrap();
                let data_len: i32 = row.get("DATA_LENGTH").unwrap();
                let row_data = ColumnData {
                    table_name,
                    table_desc,
                    column_name,
                    column_desc,
                    data_type,
                    data_len,
                };
                row_list.push(row_data);
            }
            for row in row_list {
                table_columns
                    .entry((row.table_name.clone(), row.table_desc.clone()))
                    .or_insert(Vec::new())
                    .push(row);
            }
            let mut ptable_list: Vec<TableColumnsInfo> = table_columns
                .into_iter()
                .map(
                    |((table_name, table_desc), column_infos)| TableColumnsInfo {
                        table_name,
                        table_desc,
                        column_infos,
                    },
                )
                .collect();
            ptable_list.sort_by(|a, b| a.table_name.cmp(&b.table_name));
            let db = SyncTableCmd::get_db_conn().await;
            for table in ptable_list.iter() {
                let uid = gen_uid();
                let latest_version = latest_version(table.table_name.clone()).await;
                let sync_no = gen_sync_no();
                let table_insert = sync_tables::ActiveModel {
                    id: Set(uid.clone()),
                    sync_no: Set(sync_no.clone()),
                    sync_version: Set(latest_version.clone()),
                    table_name: Set(table.table_name.clone()),
                    table_desc: Set(Some(table.table_desc.clone())),
                    created_at: Set(Some(chrono::Local::now().to_string())),
                    ..Default::default()
                };
                let inserted = sync_tables::Entity::insert(table_insert)
                    .exec(&db)
                    .await
                    .unwrap();

                for columns_info in table.column_infos.iter() {
                    let col = sync_table_columns_info::ActiveModel {
                        id: Set(gen_uid()),
                        table_name: Set(table.table_name.clone()),
                        column_name: Set(columns_info.column_name.clone()),
                        column_desc: Set(Some(columns_info.column_desc.clone())),
                        data_type: Set(columns_info.data_type.clone()),
                        data_len: Set(columns_info.data_len),
                        ref_idx: Set(uid.clone()),
                        created_at: Set(Some(chrono::Local::now().to_string())),
                        ..Default::default()
                    };
                    let _ = sync_table_columns_info::Entity::insert(col)
                        .exec(&db)
                        .await
                        .map_err(|e| eprintln!("Err: {}", e));
                }

                println!(
                    "current table {} ,sync_no:{}, version :{},id {}",
                    &table.table_name, &sync_no, &latest_version, inserted.last_insert_id
                );
            }
            ()
        }
        Err(sea_orm::DbErr::Custom(
            "no actived connection_config to use".into(),
        ))
    }

    async fn get_latest_one(db: &DatabaseConnection) -> Option<(MaxSyncNo, MaxSyncVersion)> {
        sync_tables::Entity::find()
            .select_only()
            .column(sync_tables::Column::SyncNo)
            .column_as(Expr::col(sync_tables::Column::SyncVersion).max(), "max_sv")
            .group_by(sync_tables::Column::SyncNo)
            .into_tuple::<(MaxSyncNo, MaxSyncVersion)>()
            .one(db)
            .await
            .unwrap()
    }

    pub async fn get_table_infos(
        sync_input: Option<SyncInput>,
    ) -> AppResult<Vec<SyncedTableColumnsInfo>> {
        let db = SyncTableCmd::get_db_conn().await;
        let mut data: Vec<(sync_tables::Model, Option<sync_table_columns_info::Model>)> = vec![];
        match sync_input {
            Some(sync_input) => {
                data = sync_tables::Entity::find()
                    .filter(sync_tables::Column::SyncNo.eq(sync_input.sync_no))
                    .filter(sync_tables::Column::SyncVersion.eq(sync_input.sync_version))
                    .find_also_related(SyncTableColumnsInfo)
                    .all(&db)
                    .await
                    .unwrap();
            }
            None => {
                if let Some(latest_one) = SyncTableCmd::get_latest_one(&db).await {
                    println!("latest_one:{:?}", &latest_one);
                    data = sync_tables::Entity::find()
                        .filter(sync_tables::Column::SyncNo.eq(latest_one.0))
                        .filter(sync_tables::Column::SyncVersion.eq(latest_one.1))
                        .find_also_related(SyncTableColumnsInfo)
                        .all(&db)
                        .await
                        .unwrap();
                }
            }
        }

        let table_columns_info = convert_to_table_infos_list(data);
        AppResult {
            code: 200,
            message: "success".to_string(),
            error: AppErr::None,
            data: table_columns_info,
        }
    }

    pub async fn get_table_data(query_sql: String) -> AppResult<TableRawData> {
        let cc = ConnectionConfigCmd::get_actived_config().await.unwrap();
        let source_db = DatasourceCmd::new(cc);
        let headers = extrat_headers(&query_sql);
        dbg!(&headers);
        let mut values: Vec<Vec<String>> = vec![];
        let rows = source_db
            .conn
            .query(&query_sql, &[])
            .map_err(|e| DbErr::Custom(e.to_string()))
            .unwrap();

        for (idx, row) in rows.enumerate() {
            match row {
                Ok(r) => {
                    // dbg!(&r);
                    let mut row_value_list: Vec<String> = vec![];
                    for col in headers.iter() {
                        if let Some(value) = r.get::<&str, Option<String>>(&col).unwrap_or_default()
                        {
                            row_value_list.push(value)
                        } else {
                            row_value_list.push(String::new())
                        }
                    }
                    values.push(row_value_list)
                }
                Err(e) => {
                    return AppResult {
                        code: 505,
                        message: format!("empty data at row idx: {}", idx),
                        error: AppErr::CustomErr(format!("DbErr: {}", e)),
                        data: TableRawData::new(headers, values),
                    }
                }
            }
        }
        AppResult {
            code: 200,
            message: "success".to_string(),
            error: AppErr::None,
            data: TableRawData::new(headers, values),
        }
    }

    pub async fn export_table_data(export_range: ExportRange) -> AppResult<String> {
        let db = SyncTableCmd::get_db_conn().await;
        let table_info_list: Vec<(sync_tables::Model, Option<sync_table_columns_info::Model>)>;
        match export_range.table_name {
            Some(name) => {
                table_info_list = sync_tables::Entity::find()
                    .filter(sync_tables::Column::SyncNo.eq(export_range.sync_no.clone()))
                    .filter(sync_tables::Column::SyncVersion.eq(export_range.sync_version.clone()))
                    .filter(sync_tables::Column::TableName.eq(name))
                    .find_also_related(SyncTableColumnsInfo)
                    .all(&db)
                    .await
                    .unwrap();
            }
            None => {
                table_info_list = sync_tables::Entity::find()
                    .filter(sync_tables::Column::SyncNo.eq(export_range.sync_no.clone()))
                    .filter(sync_tables::Column::SyncVersion.eq(export_range.sync_version.clone()))
                    .filter(sync_tables::Column::IsExportable.eq(true))
                    .find_also_related(SyncTableColumnsInfo)
                    .all(&db)
                    .await
                    .unwrap();
            }
        }

        println!("table_info_list len: {}", table_info_list.len());
        let table_infos = convert_to_table_infos_list(table_info_list);
        println!("table_infos len: {}", table_infos.len());

        let export_jobs = table_infos
            .iter()
            .map(|table| {
                let headers = table
                    .column_infos
                    .iter()
                    .map(|column_info| ColumnDataInput {
                        column_name: column_info.column_name.clone(),
                        data_len: column_info.data_len.clone(),
                        data_type: column_info.data_type.clone(),
                    })
                    .collect::<Vec<ColumnDataInput>>();

                let query_sql = gen_query_script(table);

                return ExportSpecInput {
                    table_name: table.table_name.clone(),
                    headers,
                    query_sql,
                };
            })
            .collect::<Vec<ExportSpecInput>>();

        let cc = ConnectionConfigCmd::get_actived_config().await.unwrap();
        let source_db = DatasourceCmd::new(cc);
        let xls = XlsxHelper::new();

        for job in export_jobs.iter() {
            println!("table : {}", job.table_name);
            let mut worksheet = xls
                .wb
                .add_worksheet(Some(&job.table_name.as_str()))
                .unwrap();
            for (col_num, column) in job.headers.iter().enumerate() {
                let _ = worksheet.write_string(
                    0,
                    col_num as u16,
                    &column.column_name,
                    Some(&XlsxHelper::headers_format()),
                );
            }

            let rows = source_db.conn.query(&job.query_sql, &[]).map_err(|err| {
                eprintln!("err:{}, table_name :{}", err, job.table_name);
            });

            match rows {
                Ok(rows) => {
                    for (row_num, row) in rows.enumerate() {
                        match row {
                            Ok(r) => {
                                for (col_num, column) in job.headers.iter().enumerate() {
                                    if let Some(value) = r
                                        .get::<&str, Option<String>>(&column.column_name)
                                        .unwrap_or(None)
                                    {
                                        // println!("{}", value);
                                        match column.data_type.as_str() {
                                            "NUMBER" => {
                                                worksheet
                                                    .write_number(
                                                        (row_num + 1) as u32,
                                                        col_num as u16,
                                                        value.parse::<f64>().unwrap(),
                                                        Some(&XlsxHelper::format()),
                                                    )
                                                    .unwrap();
                                            }
                                            _ => {
                                                worksheet
                                                    .write_string(
                                                        (row_num + 1) as u32,
                                                        col_num as u16,
                                                        &value.to_string(),
                                                        Some(&XlsxHelper::format()),
                                                    )
                                                    .unwrap();
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error fetching row: {:?}", e);
                                break;
                            }
                        }
                    }
                    // 冻结首行
                    worksheet.freeze_panes(1, 0);
                }
                Err(_) => continue,
            }
        }

        // 关闭资源句柄
        xls.wb.close().unwrap();
        let current_dir = env::current_dir().expect("Failed to get current directory");
        let file_path = current_dir
            .join(xls.file_name)
            .to_str()
            .unwrap()
            .to_string();

        AppResult {
            code: 200,
            message: "success".into(),
            error: AppErr::None,
            data: file_path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRange {
    sync_no: String,
    sync_version: i32,
    table_name: Option<String>,
}

fn gen_query_script(table: &SyncedTableColumnsInfo) -> String {
    let select_fields = table
        .column_infos
        .iter()
        .map(|col| col.column_name.to_uppercase().clone())
        .collect::<Vec<String>>()
        .join(", ");
    println!("gen_query_script fields: {select_fields}");
    format!("SELECT {} FROM {}", select_fields, table.table_name)
}

fn convert_to_table_infos_list(
    raw_data: Vec<(sync_tables::Model, Option<sync_table_columns_info::Model>)>,
) -> Vec<SyncedTableColumnsInfo> {
    let mut table_columns: HashMap<(String, i32, String, String), Vec<ColumnData>> = HashMap::new();
    for row in raw_data.iter() {
        let table = row.0.clone();
        let columns = row.1.clone().unwrap();
        let sync_no = table.sync_no.clone();
        let sync_version = table.sync_version.clone();
        let table_name: String = table.table_name.clone();
        let table_desc: String = table.table_desc.clone().unwrap_or_default();
        let column_name: String = columns.column_name.clone();
        let column_desc: String = columns.column_desc.clone().unwrap_or_default();
        let data_type: String = columns.data_type.clone();
        let data_len: i32 = columns.data_len.clone();

        table_columns
            .entry((
                sync_no.clone(),
                sync_version.clone(),
                table_name.clone(),
                table_desc.clone(),
            ))
            .or_insert(Vec::new())
            .push(ColumnData {
                table_name,
                table_desc,
                column_name,
                column_desc,
                data_type,
                data_len,
            });
    }
    let table_columns_info: Vec<SyncedTableColumnsInfo> = table_columns
        .into_iter()
        .map(
            |((sync_no, sync_version, table_name, table_desc), column_infos)| {
                let model = SyncedTableColumnsInfo::new(
                    sync_no,
                    sync_version,
                    &table_name,
                    table_desc,
                    "".to_string(),
                    column_infos,
                );
                let query_sql = gen_query_script(&model);
                SyncedTableColumnsInfo { query_sql, ..model }
            },
        )
        .collect();
    println!("total {}", table_columns_info.len());
    table_columns_info
}

type LatestVersion = i32;

async fn latest_version(table_name: String) -> LatestVersion {
    let db = SyncTableCmd::get_db_conn().await;
    let res = sync_tables::Entity::find()
        .filter(sync_tables::Column::TableName.eq(table_name))
        .order_by_desc(sync_tables::Column::SyncVersion)
        .limit(1)
        .one(&db)
        .await;
    match res.unwrap() {
        Some(table) => table.sync_version + 1,
        None => 1,
    }
}

fn gen_sync_no() -> String {
    format!("{}", chrono::Local::now().format("%Y-%m-%d").to_string())
}

fn extrat_headers(query_sql: &str) -> Vec<String> {
    println!("query_sql: {}", query_sql);
    // 使用正则表达式提取SELECT语句中的字段，包括AS别名
    let re = Regex::new(r"(?i)SELECT\s+([\s\S]*?)\s+FROM").unwrap();

    if let Some(caps) = re.captures(query_sql) {
        // 将字段按逗号分隔并去除空格
        let fields = caps[1]
            .split(',')
            .map(|field| field.trim().to_string())
            .collect::<Vec<String>>();

        // 处理AS别名
        let mut result = Vec::new();
        for field in fields {
            println!("field : {}", field);
            if field.contains("as") || field.contains("AS") {
                // 如果包含AS，取AS后面的部分作为字段
                if let Some(alias) = field.split_whitespace().last() {
                    println!("{} / {}", field, alias);
                    result.push(alias.to_string().replace("\"", ""));
                }
            } else if field.contains(".") {
                if let Some(f) = field.split(".").last() {
                    result.push(f.to_string());
                }
            } else {
                // 否则直接添加原字段
                result.push(field);
            }
        }
        return result;
    }

    Vec::new() // 如果没有匹配，返回空向量
}

#[cfg(test)]
mod tests {

    use crate::aync_tables::{gen_sync_no, ExportRange};

    use super::SyncTableCmd;
    use entity::{sync_table_columns_info::Entity as SyncTableColumnsInfo, sync_tables};
    use sea_orm::EntityTrait;
    use tokio::test;

    #[test]
    async fn test() {
        // INSERT INTO connection_config (id, env, db_type, url, username, password, is_active, abandoned_table_list, created_at) VALUES('A5xZqA', 'Test', 'Oracle', '10.0.1.202:1521/testdb', 'mesadm', 'Zeta_2024', 1, NULL, '2024-09-21 15:54:38.945347 +08:00');
        let _ = SyncTableCmd::sync_table_infos().await;
    }

    #[test]
    async fn test_gen_sync_no() {
        let no = gen_sync_no();
        println!("{}", no);
    }

    #[test]
    async fn test_query() {
        let db = SyncTableCmd::get_db_conn().await;

        let data = sync_tables::Entity::find_by_id("LwKWYa")
            .find_also_related(SyncTableColumnsInfo)
            .all(&db)
            .await
            .unwrap();

        for d in data.iter() {
            let column = d.1.clone().unwrap().column_name;
            println!("{:?},{},{:?}", d.0.table_name, d.0.sync_version, column);
            dbg!(&d);
        }
    }

    #[test]
    async fn test_get_table_infos() {
        let data = SyncTableCmd::get_table_data(
            "SELECT id,sync_no,sync_version, COUNT(sync_version) as cnt from sync_tables "
                .to_owned(),
        )
        .await;
        dbg!(&data);
    }

    #[test]
    async fn test_export_table_data1() {
        let et = ExportRange {
            sync_no: "2024-09-21".into(),
            sync_version: 1,
            table_name: None,
        };
        let start_time = chrono::Local::now();
        let _ = SyncTableCmd::export_table_data(et).await;
        let end_time = chrono::Local::now();
        let duration = end_time - start_time;
        let duration_minutes = duration.num_seconds();

        println!("Time taken: {} minutes", duration_minutes);
    }

    #[test]
    async fn test_export_table_data2() {
        let et = ExportRange {
            sync_no: "2024-09-21".into(),
            sync_version: 1,
            table_name: Some("ALARMS".into()),
        };
        let start_time = chrono::Local::now();
        let _ = SyncTableCmd::export_table_data(et).await;
        let end_time = chrono::Local::now();
        let duration = end_time - start_time;
        let duration_minutes = duration.num_seconds();

        println!("Time taken: {} minutes", duration_minutes);
    }
}
