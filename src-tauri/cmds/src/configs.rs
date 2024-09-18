use common::{
    constants::DATABASE_URL, models::input::AbandedTableNameInput, utils::remove_target_with_commas,
};
use entity::connection_config::{self, Entity as ConnectionConfig};
use sea_orm::{
    ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, Order, QueryFilter, QueryOrder, Set,
};

pub struct ConnectionConfigCmd;

impl ConnectionConfigCmd {
    pub fn new() -> Self {
        Self
    }

    async fn get_db_conn() -> DbConn {
        let db = sea_orm::Database::connect(DATABASE_URL)
            .await
            .expect("can not connect to database, how old are you!");
        db
    }

    pub async fn get_all_configs() -> Result<Vec<connection_config::Model>, DbErr> {
        let db = Self::get_db_conn().await;
        ConnectionConfig::find()
            .order_by(connection_config::Column::CreatedAt, Order::Desc)
            .all(&db)
            .await
    }

    pub async fn get_actived_config() -> Option<connection_config::Model> {
        let db = Self::get_db_conn().await;
        ConnectionConfig::find()
            .filter(connection_config::Column::IsActive.eq(true))
            .one(&db)
            .await
            .unwrap()
    }

    pub async fn new_config(
        form_data: connection_config::Model,
    ) -> Result<connection_config::Model, DbErr> {
        let db = Self::get_db_conn().await;
        let act_model = connection_config::ActiveModel {
            id: Set(form_data.id),
            db_type: Set(form_data.db_type),
            env: Set(form_data.env),
            url: Set(form_data.url),
            username: Set(form_data.username),
            password: Set(form_data.password),
            is_active: Set(form_data.is_active),
            created_at: Set(form_data.created_at),
            ..Default::default()
        };

        let res = ConnectionConfig::insert(act_model).exec(&db).await?;

        Ok(ConnectionConfig::find_by_id(res.last_insert_id)
            .one(&db)
            .await?
            .unwrap())
    }

    pub async fn delete_config_by_id(id: String) -> Result<DeleteResult, DbErr> {
        let db = Self::get_db_conn().await;

        let connection_config: connection_config::ActiveModel = ConnectionConfig::find_by_id(id)
            .one(&db)
            .await?
            .ok_or(DbErr::Custom("can not find connection_config".to_owned()))
            .map(Into::into)?;

        ConnectionConfig::delete(connection_config).exec(&db).await
    }

    /// always left one connection_config actived
    pub async fn active_config_by_id(id: String) -> Result<connection_config::Model, DbErr> {
        let db = Self::get_db_conn().await;

        // all set false
        ConnectionConfig::update_many()
            .col_expr(connection_config::Column::IsActive, false.into())
            .exec(&db)
            .await?;

        // active by id
        let mut connection_config: connection_config::ActiveModel =
            ConnectionConfig::find_by_id(id)
                .one(&db)
                .await?
                .ok_or(DbErr::Custom("can not find connection_config".to_owned()))
                .map(Into::into)?;

        connection_config.is_active = Set(true);

        ConnectionConfig::update(connection_config).exec(&db).await
    }

    pub async fn update_aband_table_list(
        db: &DbConn,
        banned_table_input: AbandedTableNameInput,
    ) -> Result<connection_config::Model, DbErr> {
        let mut connection_config: connection_config::ActiveModel = ConnectionConfig::find()
            .filter(connection_config::Column::IsActive.eq(true))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(
                "can not find actived connection_config in db".to_owned(),
            ))
            .map(Into::into)?;

        if banned_table_input.table_name.is_empty() {
            return Err(DbErr::Custom(format!(
                "can not update ban_table_list with empty value: {}",
                banned_table_input.table_name
            )));
        }

        let bti_with_commas = format!("'{}'", banned_table_input.table_name.clone());

        let mut banned_tables_str = connection_config
            .abandoned_table_list
            .unwrap()
            .unwrap_or_else(|| String::new());

        let cloned_bt_list = banned_tables_str.clone();
        let banned_tables: Vec<&str> = cloned_bt_list.split(',').collect();

        if banned_tables.contains(&bti_with_commas.as_str()) {
            // 如果banned_table_input.table_name已经存在，将其移除
            banned_tables_str = remove_target_with_commas(banned_tables.clone(), &bti_with_commas);
        } else {
            // 新增
            banned_tables_str.push_str(format!(",'{}'", banned_table_input.table_name).as_str());
        }

        // ban_table_list有可能为空
        connection_config.abandoned_table_list = Set(if banned_tables.is_empty() {
            None
        } else {
            Some(banned_tables_str)
        });
        ConnectionConfig::update(connection_config).exec(db).await
    }

    pub async fn reset_banned_table_list() -> Result<connection_config::Model, DbErr> {
        let db = Self::get_db_conn().await;
        let mut connection_config: connection_config::ActiveModel = ConnectionConfig::find()
            .filter(connection_config::Column::IsActive.eq(true))
            .one(&db)
            .await?
            .ok_or(DbErr::Custom(
                "can not find actived connection_config in db".to_owned(),
            ))
            .map(Into::into)?;

        connection_config.abandoned_table_list = Set(None);
        ConnectionConfig::update(connection_config).exec(&db).await
    }
}
