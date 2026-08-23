use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsersRoles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UsersRoles::UserId).integer().not_null())
                    .col(ColumnDef::new(UsersRoles::RoleId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .table(UsersRoles::Table)
                            .col(UsersRoles::UserId)
                            .col(UsersRoles::RoleId),
                    )
                    .to_owned(),
            )
            .await?;

        // Remove the single-role column now that the many-to-many table exists.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::RoleId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::RoleId).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UsersRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UsersRoles {
    Table,
    UserId,
    RoleId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    RoleId,
}
