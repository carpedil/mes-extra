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
            get_table_infos,
            get_table_data,
            export_table_data,
            get_product_def_list,
            get_product_def_ver_list,
            get_process_flow_list,
            get_process_flow_ver_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    #[test]
    fn test_count_lines_in_rs_files() {
        let dir_path = "../src-tauri"; // 替换为你要统计的目录
        let total_lines = count_lines_in_rs_files(dir_path);
        println!("Total lines of Rust code: {}", total_lines);
    }

    fn count_lines_in_rs_files<P: AsRef<Path>>(dir: P) -> usize {
        let mut total_lines = 0;

        // 读取目录中的所有条目
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                // 如果是目录，则递归调用
                if path.is_dir() {
                    // 排除target目录
                    if path.file_name().and_then(|s| s.to_str()) != Some("target") {
                        total_lines += count_lines_in_rs_files(&path);
                    }
                } else if let Some(extension) = path.extension() {
                    // 统计.rs文件的行数
                    if extension == "rs" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let current_lines = content.lines().count();
                            total_lines += current_lines;
                            println!(
                                "| current lines:{} \t| total_lines:{} \t| path: {:?} | {:#?} ",
                                current_lines,
                                total_lines,
                                path,
                                path.file_name().unwrap(),
                            );
                        }
                    }
                }
            }
        }

        total_lines
    }
}
