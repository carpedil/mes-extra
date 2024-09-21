use sea_orm_migration::prelude::*;

// sea-orm-cli migrate refresh -u "sqlite:./mes-extra.db?mode=rwc"
#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
