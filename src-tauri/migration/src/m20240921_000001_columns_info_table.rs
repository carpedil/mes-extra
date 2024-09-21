use sea_orm_migration::prelude::*;

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
                            .integer()
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
                    .col(ColumnDef::new(SyncTableColumnsInfo::DataType).string_len(30))
                    .col(ColumnDef::new(SyncTableColumnsInfo::DataLen).integer())
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::IsExportable)
                            .boolean()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SyncTableColumnsInfo::SortType)
                            .string_len(10)
                            .default("ASC"),
                    )
                    .col(ColumnDef::new(SyncTableColumnsInfo::CreatedAt).timestamp())
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
    DataType,
    DataLen,
    IsExportable,
    SortType,
    CreatedAt,
}
