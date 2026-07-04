use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("agents"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("wg_tunnel_ip")).string().null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("agents"))
                    .drop_column(Alias::new("wg_tunnel_ip"))
                    .to_owned(),
            )
            .await
    }
}
