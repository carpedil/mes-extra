use super::m20240921_000001_columns_info_table::SyncTableColumnsInfo;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SyncTables::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SyncTables::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SyncTables::SyncNo).string_len(30).not_null())
                    .col(
                        ColumnDef::new(SyncTables::SyncVersion)
                            .string_len(5)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTables::TableName)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncTables::IsExportable)
                            .boolean()
                            .default(true),
                    )
                    .col(ColumnDef::new(SyncTables::FkColumnsId).integer())
                    .col(ColumnDef::new(SyncTables::CreatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-columns-id")
                            .from(SyncTables::Table, SyncTables::FkColumnsId)
                            .to(SyncTableColumnsInfo::Table, SyncTableColumnsInfo::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SyncTables::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SyncTables {
    Table,
    Id,
    SyncNo,
    SyncVersion,
    TableName,
    IsExportable,
    FkColumnsId,
    CreatedAt,
}
