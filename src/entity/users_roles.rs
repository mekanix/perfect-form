use super::{role, user};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: BelongsTo<user::Entity>,
    #[sea_orm(belongs_to, from = "role_id", to = "id")]
    pub role: BelongsTo<role::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
