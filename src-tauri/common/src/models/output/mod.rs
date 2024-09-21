use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DbTableStruct {
    pub table_name: String,
    pub table_desc: String,
    pub column_name: String,
    pub column_desc: String,
    pub data_type: String,
    pub data_len: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableColumnsInfo {
    pub table_name: String,
    pub table_desc: String,
    pub column_infos: Vec<ColumnData>,
}

impl TableColumnsInfo {
    pub fn new(table_name: &str, table_desc: String, column_infos: Vec<ColumnData>) -> Self {
        Self {
            table_name: table_name.to_owned(),
            table_desc,
            column_infos,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnData {
    pub table_name: String,
    pub table_desc: String,
    pub column_name: String,
    pub column_desc: String,
    pub data_type: String,
    pub data_len: i32,
}

impl ColumnData {
    pub fn new(
        table_name: String,
        table_desc: String,
        column_name: String,
        column_desc: String,
        data_type: String,
        data_len: i32,
    ) -> Self {
        Self {
            table_name,
            table_desc,
            column_name,
            column_desc,
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
