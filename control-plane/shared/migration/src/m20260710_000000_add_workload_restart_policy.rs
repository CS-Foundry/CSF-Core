use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("workloads"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("restart_policy"))
                            .string()
                            .not_null()
                            .default("always"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("max_restarts")).integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("restart_count"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("workloads"))
                    .drop_column(Alias::new("restart_policy"))
                    .drop_column(Alias::new("max_restarts"))
                    .drop_column(Alias::new("restart_count"))
                    .to_owned(),
            )
            .await
    }
}
