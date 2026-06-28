use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_resource_groups_cidr_unique")
                    .table(Alias::new("resource_groups"))
                    .col(Alias::new("internal_cidr"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("agents"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("wg_public_key"))
                            .text()
                            .null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("wg_endpoint"))
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_resource_groups_cidr_unique")
                    .table(Alias::new("resource_groups"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("agents"))
                    .drop_column(Alias::new("wg_public_key"))
                    .drop_column(Alias::new("wg_endpoint"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
