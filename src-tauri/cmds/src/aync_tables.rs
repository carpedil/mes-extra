use std::collections::HashMap;

use common::{
    constants::DATABASE_URL,
    output::{ColumnData, DbTableStruct, TableColumnsInfo},
    utils::{gen_uid, get_user_tab_columns_sql},
};
use entity::{
    sync_table_columns_info::{self},
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
        let mut table_columns: HashMap<String, Vec<ColumnData>> = HashMap::new();
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
                let column_name: String = row.get("COLUMN_NAME").unwrap();
                let data_type: String = row.get("DATA_TYPE").unwrap();
                let data_len: i32 = row.get("DATA_LENGTH").unwrap();
                let row_data = DbTableStruct {
                    table_name,
                    column_name,
                    data_type,
                    data_len,
                };
                row_list.push(row_data);
            }
            for row in row_list {
                table_columns
                    .entry(row.table_name.clone())
                    .or_insert(Vec::new())
                    .push(ColumnData {
                        table_name: row.table_name,
                        column_name: row.column_name,
                        data_type: row.data_type,
                        data_len: row.data_len,
                    });
            }
            let mut ptable_list: Vec<TableColumnsInfo> = table_columns
                .into_iter()
                .map(|(table_name, column_infos)| TableColumnsInfo {
                    table_name,
                    column_infos,
                })
                .collect();
            ptable_list.sort_by(|a, b| a.table_name.cmp(&b.table_name));
            let db = SyncTableCmd::get_db_conn().await;
            for table in ptable_list.iter() {
                let uid = gen_uid();
                let latest_version = get_latest_version(table.table_name.clone()).await;
                let sync_no = gen_sync_no();
                let table_insert = sync_tables::ActiveModel {
                    id: Set(uid.clone()),
                    sync_no: Set(sync_no.clone()),
                    sync_version: Set(latest_version.clone()),
                    table_name: Set(table.table_name.clone()),
                    created_at: Set(Some(chrono::Local::now().to_string())),
                    ..Default::default()
                };
                let inserted = sync_tables::Entity::insert(table_insert)
                    .exec(&db)
                    .await
                    .unwrap();
                //
                for columns_info in table.column_infos.iter() {
                    let col = sync_table_columns_info::ActiveModel {
                        id: Set(gen_uid()),
                        table_name: Set(table.table_name.clone()),
                        column_name: Set(columns_info.column_name.clone()),
                        data_type: Set(Some(columns_info.data_type.clone())),
                        data_len: Set(Some(columns_info.data_len)),
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
}

async fn get_latest_version(table_name: String) -> i32 {
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
}
