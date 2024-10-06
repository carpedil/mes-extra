use entity::connection_config::{self};
use oracle::Connection;

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
}
