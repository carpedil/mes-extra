use std::collections::HashMap;

use common::{
    constants::DATABASE_URL,
    output::{AppErr, AppResult, ColumnData, DbTableStruct, TableColumnsInfo, TableRawData},
    utils::{gen_uid, get_user_tab_columns_sql},
};
use entity::{
    sync_table_columns_info::{self, Entity as SyncTableColumnsInfo},
    sync_tables::{self},
};
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};

use crate::{ConnectionConfigCmd, DatasourceCmd};

pub struct SyncTableCmd;

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
            let mut row_list: Vec<DbTableStruct> = vec![];
            for row in data.into_iter() {
                let row = row.unwrap();
                let table_name: String = row.get("TABLE_NAME").unwrap();
                let table_desc: String = row.get("TAB_DESC").unwrap_or("".to_string());
                let column_name: String = row.get("COLUMN_NAME").unwrap();
                let column_desc: String = row.get("COL_DESC").unwrap_or("".to_string());
                let data_type: String = row.get("DATA_TYPE").unwrap();
                let data_len: i32 = row.get("DATA_LENGTH").unwrap();
                let row_data = DbTableStruct {
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
                    .push(ColumnData {
                        table_name: row.table_name,
                        table_desc: row.table_desc,
                        column_name: row.column_name,
                        column_desc: row.column_desc,
                        data_type: row.data_type,
                        data_len: row.data_len,
                    });
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

    pub async fn get_table_infos(
        sync_no: String,
        sync_version: i32,
    ) -> AppResult<Vec<TableColumnsInfo>> {
        let db = SyncTableCmd::get_db_conn().await;

        let data = sync_tables::Entity::find()
            .filter(sync_tables::Column::SyncNo.eq(sync_no))
            .filter(sync_tables::Column::SyncVersion.eq(sync_version))
            .find_also_related(SyncTableColumnsInfo)
            .all(&db)
            .await
            .unwrap();
        let table_columns_info = convert_to_table_infos_list(data);
        AppResult {
            code: 200,
            message: "success".to_string(),
            error: AppErr::None,
            data: table_columns_info,
        }
    }

    pub async fn get_table_data(
        sync_no: String,
        sync_version: i32,
        table_name: String,
    ) -> AppResult<Vec<TableRawData>> {
        let db = SyncTableCmd::get_db_conn().await;
        let table_info = sync_tables::Entity::find()
            .filter(sync_tables::Column::SyncNo.eq(sync_no))
            .filter(sync_tables::Column::SyncVersion.eq(sync_version))
            .filter(sync_tables::Column::TableName.eq(table_name))
            .find_also_related(SyncTableColumnsInfo)
            .all(&db)
            .await;

        let mut data_list: Vec<TableRawData> = vec![];
        match table_info {
            Ok(data) => {
                if &data.is_empty() == &true {
                    return AppResult {
                        code: 404,
                        message: "Resource is not reachable".into(),
                        error: AppErr::DbErr(format!("no data found ")),
                        data: data_list,
                    };
                }
                let table_infos = convert_to_table_infos_list(data);
                dbg!(&table_infos);
                let sql = gen_query_script(&table_infos[0]);
                let headers = &table_infos[0]
                    .column_infos
                    .iter()
                    .map(|col| col.column_name.clone())
                    .collect::<Vec<String>>();
                let cc = ConnectionConfigCmd::get_actived_config().await.unwrap();
                let source_db = DatasourceCmd::new(cc);

                let rows = source_db
                    .conn
                    .query(&sql, &[])
                    .map_err(|e| DbErr::Custom(e.to_string()))
                    .unwrap();

                for (idx, row) in rows.enumerate() {
                    match row {
                        Ok(r) => {
                            let mut row_value_list: Vec<String> = vec![];
                            for (_, col) in headers.iter().enumerate() {
                                if let Some(value) =
                                    r.get::<&str, Option<String>>(&col).unwrap_or_default()
                                {
                                    row_value_list.push(value)
                                } else {
                                    row_value_list.push(String::new())
                                }
                            }
                            data_list.push(TableRawData::new(row_value_list))
                        }
                        Err(e) => {
                            return AppResult {
                                code: 505,
                                message: format!("empty data at row idx: {}", idx),
                                error: AppErr::CustomErr(format!("DbErr: {}", e)),
                                data: data_list,
                            }
                        }
                    }
                }
                AppResult {
                    code: 200,
                    message: "success".to_string(),
                    error: AppErr::None,
                    data: data_list,
                }
            }
            Err(e) => {
                return AppResult {
                    code: 404,
                    message: "data not found".into(),
                    data: data_list,
                    error: AppErr::CustomErr(format!("data not  found,{:?}", e)),
                }
            }
        }
    }
}

fn gen_query_script(table: &TableColumnsInfo) -> String {
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
) -> Vec<TableColumnsInfo> {
    let mut table_columns: HashMap<(String, String), Vec<ColumnData>> = HashMap::new();
    for row in raw_data.iter() {
        let table = row.0.clone();
        let columns = row.1.clone().unwrap();
        let table_name: String = table.table_name.clone();
        let table_desc: String = table.table_desc.unwrap().clone();
        let column_name: String = columns.column_name.clone();
        let column_desc: String = columns.column_desc.clone().unwrap_or_default();
        let data_type: String = columns.data_type.clone();
        let data_len: i32 = columns.data_len.clone();

        table_columns
            .entry((table_name.clone(), table_desc.clone()))
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
    let table_columns_info: Vec<TableColumnsInfo> = table_columns
        .into_iter()
        .map(
            |((table_name, table_desc), column_infos)| TableColumnsInfo {
                table_name,
                table_desc,
                column_infos,
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

#[cfg(test)]
mod tests {

    use crate::aync_tables::gen_sync_no;

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
        let data =
            SyncTableCmd::get_table_data("2024-09-21".to_string(), 1, "ALARMS".to_string()).await;
        dbg!(&data);
    }
}
