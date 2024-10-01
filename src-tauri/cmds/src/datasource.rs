use std::{env, ops::Mul};

use common::{excel_helper::XlsxHelper, input::ExportSpecInput, output::TableRawData};
use entity::connection_config::{self};
use oracle::Connection;
use sea_orm::DbErr;

use crate::ConnectionConfigCmd;

pub struct DatasourceCmd {
    pub conn: Connection,
}

#[allow(dead_code)]
impl DatasourceCmd {
    pub fn new(connection_config: connection_config::Model) -> Self {
        let conn = Connection::connect(
            connection_config.username,
            connection_config.password,
            connection_config.url,
        )
        .expect("can not connect to datasource db check your network setting");
        Self { conn }
    }

    pub async fn dump_datasource_tables(dump_spec: Vec<ExportSpecInput>) -> Result<String, DbErr> {
        if let Some(connection_config) = ConnectionConfigCmd::get_actived_config().await {
            println!("active_tcc: {}", connection_config.id);
            let dq = DatasourceCmd::new(connection_config);
            let xls_helper = XlsxHelper::new();

            for dump_spec in dump_spec.into_iter() {
                println!("table : {}", dump_spec.table_name);
                let mut worksheet = xls_helper
                    .wb
                    .add_worksheet(Some(&dump_spec.table_name.as_str()))
                    .unwrap();
                // write columns
                for (col_num, column) in dump_spec.headers.into_iter().enumerate() {
                    let _ = worksheet.write_string(
                        0,
                        col_num as u16,
                        &column.column_name,
                        Some(&XlsxHelper::headers_format()),
                    );
                }

                let rows = dq.conn.query(&dump_spec.query_sql, &[]).expect(
                    format!(
                        "Error fetching rows,table_name:{},sql:{}",
                        &dump_spec.table_name, &dump_spec.query_sql
                    )
                    .as_str(),
                );

                let column_info: Vec<(String, &oracle::sql_type::OracleType)> = rows
                    .column_info()
                    .iter()
                    .map(|ci| (ci.name().to_string(), ci.oracle_type()))
                    .collect();

                let rows = dq
                    .conn
                    .query(&dump_spec.query_sql, &[])
                    .expect("Error fetching rows");
                // 计算每列的最大宽度
                let mut col_widths = vec![0; column_info.len()];
                for row in rows {
                    match row {
                        Ok(r) => {
                            for (col_num, (col, _col_type)) in column_info.iter().enumerate() {
                                if let Some(value) =
                                    r.get::<&str, Option<String>>(&col).unwrap_or(None)
                                {
                                    let v1 = &value.chars().count();
                                    let v2 = col_widths[col_num];
                                    if !col_widths.contains(v1) || !col_widths.contains(&v2) {
                                        if v1 > &v2 {
                                            // println!("col_num = {}|v2 = {}|v1 = {}|col_name = {}|value = {}", col_num,v2, v1, col,value);
                                            col_widths[col_num] = *v1;
                                            continue;
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
                dbg!(&col_widths.clone());
                let rows = dq
                    .conn
                    .query(&dump_spec.query_sql, &[])
                    .expect("Error fetching rows");

                for (row_num, row) in rows.enumerate() {
                    match row {
                        Ok(r) => {
                            for (col_num, (col, col_type)) in column_info.iter().enumerate() {
                                if let Some(value) =
                                    r.get::<&str, Option<String>>(&col).unwrap_or(None)
                                {
                                    // println!("{}", value);
                                    match col_type {
                                        oracle::sql_type::OracleType::Number(_, _) => {
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
                for (colum_num, widths) in col_widths.clone().into_iter().enumerate() {
                    println!("第{}列 ,widths = {} ", colum_num, widths);
                    worksheet
                        .set_column(
                            colum_num as u16,
                            colum_num as u16,
                            (widths as f64).mul(1.4),
                            Some(&XlsxHelper::format()),
                        )
                        .unwrap();
                }
                // 冻结首行
                worksheet.freeze_panes(1, 0);
            }
            // 关闭资源句柄
            xls_helper.wb.close().unwrap();
            let current_dir = env::current_dir().expect("Failed to get current directory");
            let file_path = current_dir.join(xls_helper.file_name);
            return Ok(file_path.to_str().unwrap().to_string());
        }
        Err(sea_orm::DbErr::Custom(
            "no actived connection_config to use".into(),
        ))
    }

    pub async fn get_table_data(input: ExportSpecInput) -> Result<Vec<TableRawData>, DbErr> {
        if let Some(connection_config) = ConnectionConfigCmd::get_actived_config().await {
            let dqs = DatasourceCmd::new(connection_config);
            let rows = dqs
                .conn
                .query(&input.query_sql, &[])
                .map_err(|e| DbErr::Custom(e.to_string()))?;

            // let _pt = TableColumnsInfo::new(&input.table_name, "".into(), vec![]);
            let mut data_list: Vec<TableRawData> = vec![];
            for (_, row) in rows.enumerate() {
                match row {
                    Ok(r) => {
                        // let mut data: BTreeMap<String, String> = BTreeMap::new();
                        let mut row_value_list: Vec<String> = vec![];
                        for (_, col) in input.headers.iter().enumerate() {
                            if let Some(value) = r
                                .get::<&str, Option<String>>(&col.column_name)
                                .unwrap_or_default()
                            {
                                // data.insert(col.to_string(), value);
                                row_value_list.push(value)
                            } else {
                                row_value_list.push(String::new())
                            }
                        }
                        data_list.push(TableRawData::new(row_value_list))
                    }
                    Err(e) => return Err(sea_orm::DbErr::Custom(e.to_string())),
                }
            }
            return Ok(data_list);
        }
        Err(sea_orm::DbErr::Custom(
            "no actived connection_config to use".into(),
        ))
    }
}
