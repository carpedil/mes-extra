use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DbTableStruct {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub data_len: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableColumnsInfo {
    pub table_name: String,
    pub column_infos: Vec<ColumnData>,
}

impl TableColumnsInfo {
    pub fn new(table_name: &str, column_infos: Vec<ColumnData>) -> Self {
        Self {
            table_name: table_name.to_owned(),
            column_infos,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnData {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub data_len: i32,
}

impl ColumnData {
    pub fn new(table_name: String, column_name: String, data_type: String, data_len: i32) -> Self {
        Self {
            table_name,
            column_name,
            data_type,
            data_len,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableRawData {
    // pub data: BTreeMap<String,String>
    pub data: Vec<String>,
}

impl TableRawData {
    pub fn new(data: Vec<String>) -> Self {
        Self { data }
    }
}
