use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::Table)
                    .if_not_exists()
                    .col(string(RefreshToken::Token).primary_key())
                    .col(integer(RefreshToken::UserId))
                    .col(date_time(RefreshToken::ExpiresAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_token_user")
                            .from(RefreshToken::Table, RefreshToken::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
    Token,
    UserId,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
