#[derive(Debug, toasty::Model)]
#[table = "users"]
pub struct Model {
    #[key]
    #[auto]
    pub id: i32,
    #[unique]
    pub email: String,
    pub password_hash: String,
    pub admin: bool,
    #[has_many(pair = user)]
    pub memberships: toasty::Deferred<Vec<super::users_roles::Model>>,
    #[has_many(via = memberships.role)]
    pub roles: toasty::Deferred<Vec<super::role::Model>>,
}
