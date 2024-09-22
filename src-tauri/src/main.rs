// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use cmd::*;
use cmds::sea_orm;
use common::constants::DATABASE_URL;
use migration::MigratorTrait;
use tauri::Manager;
mod cmd;

#[tokio::main]
async fn main() {
    let connection = sea_orm::Database::connect(DATABASE_URL)
        .await
        .expect("msg: connection is not established");
    migration::Migrator::up(&connection, None)
        .await
        .expect("Migrations failed");
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.maximize()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            new_config,
            get_all_configs,
            delete_config_by_id,
            active_config_by_id,
            load_datasource_tables,
            dump_datasource_tables,
            get_table_data,
            get_table_infos,
            get_table_data2,
            export_table_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
