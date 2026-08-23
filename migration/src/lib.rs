pub use sea_orm_migration::prelude::*;

mod m20260820_222818_add_roles;
mod m20260820_223028_create_user_roles;
mod m20260821_000001_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260821_000001_create_users_table::Migration),
            Box::new(m20260820_222818_add_roles::Migration),
            Box::new(m20260820_223028_create_user_roles::Migration),
        ]
    }
}
