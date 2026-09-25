use super::{role, user};

#[derive(Debug, toasty::Model)]
#[table = "users_roles"]
#[key(user_id, role_id)]
pub struct Model {
    #[index]
    pub user_id: i32,
    #[index]
    pub role_id: i32,
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::Deferred<user::Model>,
    #[belongs_to(key = role_id, references = id)]
    pub role: toasty::Deferred<role::Model>,
}
