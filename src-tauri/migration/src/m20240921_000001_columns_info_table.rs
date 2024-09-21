use sea_orm_migration::prelude::*;

use super::m20240921_000001_sync_tables_table::SyncTables;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SyncTableColumnsInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::Id)
                            .string_len(10)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::TableName)
                            .string_len(5)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::ColumnName)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SyncTableColumnsInfo::ColumnDesc).string_len(90))
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::DataType)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::DataLen)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::IsExportable)
                            .boolean()
                            .default(true)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::SortType)
                            .string_len(10)
                            .default("ASC")
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::RefIdx)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SyncTableColumnsInfo::CreatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-columns-idx")
                            .from(SyncTableColumnsInfo::Table, SyncTableColumnsInfo::RefIdx)
                            .to(SyncTables::Table, SyncTables::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SyncTableColumnsInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SyncTableColumnsInfo {
    Table,
    Id,
    TableName,
    ColumnName,
    ColumnDesc,
    DataType,
    DataLen,
    IsExportable,
    SortType,
    RefIdx,
    CreatedAt,
}
