use common::output::AppResult;
use entity::connection_config;

use cmds::datasource::DatasourceCmd;
use cmds::{aync_tables::SyncTableCmd, configs::ConnectionConfigCmd};
use common::{
    input::ExportSpecInput,
    models::input::CreateConnectionConfigInput,
    output::{TableColumnsInfo, TableRawData},
};

#[tauri::command]
pub async fn new_config(config: CreateConnectionConfigInput) -> connection_config::Model {
    dbg!(&config);
    let data = ConnectionConfigCmd::new_config(config.into_model_with_arbitrary_id()).await;
    data.unwrap()
}

#[tauri::command]
pub async fn get_all_configs() -> Vec<connection_config::Model> {
    let data = ConnectionConfigCmd::get_all_configs().await;
    data.unwrap()
}

#[tauri::command]
pub async fn delete_config_by_id(id: String) -> u64 {
    ConnectionConfigCmd::delete_config_by_id(id)
        .await
        .unwrap()
        .rows_affected
}

#[tauri::command]
pub async fn active_config_by_id(id: String) -> connection_config::Model {
    ConnectionConfigCmd::active_config_by_id(id).await.unwrap()
}

#[tauri::command]
pub async fn load_datasource_tables() -> Vec<TableColumnsInfo> {
    DatasourceCmd::load_datasource_tables().await.unwrap()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn dump_datasource_tables(dump_spec: Vec<ExportSpecInput>) -> String {
    dbg!(&dump_spec);
    DatasourceCmd::dump_datasource_tables(dump_spec)
        .await
        .unwrap()
}

#[tauri::command]
pub async fn get_table_data(input: ExportSpecInput) -> Vec<TableRawData> {
    DatasourceCmd::get_table_data(input).await.unwrap()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_table_infos(
    sync_no: String,
    sync_version: i32,
) -> AppResult<Vec<TableColumnsInfo>> {
    SyncTableCmd::get_table_infos(sync_no, sync_version).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_table_data2(
    sync_no: String,
    sync_version: i32,
    table_name: String,
) -> AppResult<Vec<TableRawData>> {
    SyncTableCmd::get_table_data(sync_no, sync_version, table_name).await
}
