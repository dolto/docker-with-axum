use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::GoogleOauth)
                            .unique_key()
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Users::NaverOauth)
                            .unique_key()
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Users::KakaoOauth)
                            .unique_key()
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Users::GitHubOauth)
                            .unique_key()
                            .string()
                            .null(),
                    )
                    .drop_column(Users::Password)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::GoogleOauth)
                    .drop_column(Users::NaverOauth)
                    .drop_column(Users::KakaoOauth)
                    .drop_column(Users::GitHubOauth)
                    .add_column(
                        ColumnDef::new(Users::Password)
                            .string()
                            .not_null()
                            .default("unknown"),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    GoogleOauth,
    GitHubOauth,
    NaverOauth,
    KakaoOauth,
    Password,
}
