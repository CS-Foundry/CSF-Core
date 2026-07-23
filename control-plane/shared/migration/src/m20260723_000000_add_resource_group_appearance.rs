use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("resource_groups"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("icon"))
                            .string()
                            .not_null()
                            .default("mdi:cube-outline"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("color"))
                            .string()
                            .not_null()
                            .default("#6366f1"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("pinned"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("resource_groups"))
                    .drop_column(Alias::new("icon"))
                    .drop_column(Alias::new("color"))
                    .drop_column(Alias::new("pinned"))
                    .to_owned(),
            )
            .await
    }
}
