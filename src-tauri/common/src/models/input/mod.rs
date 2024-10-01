use entity::connection_config;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

pub type FilePath = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncInput {
    pub sync_no: String,
    pub sync_version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSpecInput {
    pub table_name: String,
    pub headers: Vec<ColumnDataInput>,
    pub query_sql: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnDataInput {
    pub column_name: String,
    pub data_type: String,
    pub data_len: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConnectionConfigInput {
    pub db_type: String,
    pub env: String,
    pub database_url: String,
    pub username: String,
    pub password: String,
    pub is_active: bool,
}

impl CreateConnectionConfigInput {
    pub fn into_model_with_arbitrary_id(self) -> connection_config::Model {
        connection_config::Model {
            id: nanoid!(6),
            db_type: self.db_type,
            env: self.env,
            url: self.database_url,
            username: self.username,
            password: self.password,
            is_active: self.is_active,
            abandoned_table_list: None,
            created_at: Some(chrono::Local::now().to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MutationResult {
    pub rows_affected: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbandedTableNameInput {
    pub table_name: String,
}
