use cmds::aync_tables::ExportRange;
use common::input::SyncInput;
use common::output::{AppResult, SyncedTableColumnsInfo};
use entity::connection_config;

use cmds::datasource::DatasourceCmd;
use cmds::{aync_tables::SyncTableCmd, configs::ConnectionConfigCmd};
use common::{
    input::ExportSpecInput, models::input::CreateConnectionConfigInput, output::TableRawData,
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

#[tauri::command(rename_all = "snake_case")]
pub async fn dump_datasource_tables(dump_spec: Vec<ExportSpecInput>) -> String {
    dbg!(&dump_spec);
    DatasourceCmd::dump_datasource_tables(dump_spec)
        .await
        .unwrap()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_table_infos(
    sync_input: Option<SyncInput>,
) -> AppResult<Vec<SyncedTableColumnsInfo>> {
    SyncTableCmd::get_table_infos(sync_input).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_table_data(query_sql: String) -> AppResult<TableRawData> {
    SyncTableCmd::get_table_data(query_sql).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_table_data(export_range: ExportRange) -> AppResult<String> {
    SyncTableCmd::export_table_data(export_range).await
}
