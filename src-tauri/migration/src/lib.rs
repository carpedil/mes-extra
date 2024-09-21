pub use sea_orm_migration::prelude::*;

mod m20240916_000001_connection_config_table;
mod m20240921_000001_columns_info_table;
mod m20240921_000001_sync_tables_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240916_000001_connection_config_table::Migration),
            Box::new(m20240921_000001_sync_tables_table::Migration),
            Box::new(m20240921_000001_columns_info_table::Migration),
        ]
    }
}
