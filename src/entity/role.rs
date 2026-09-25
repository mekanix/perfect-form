#[derive(Debug, toasty::Model)]
#[table = "roles"]
pub struct Model {
    #[key]
    #[auto]
    pub id: i32,
    #[unique]
    pub name: String,
    #[has_many(pair = role)]
    pub memberships: toasty::Deferred<Vec<super::users_roles::Model>>,
    #[has_many(via = memberships.user)]
    pub users: toasty::Deferred<Vec<super::user::Model>>,
}
