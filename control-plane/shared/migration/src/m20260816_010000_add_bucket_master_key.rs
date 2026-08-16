use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("buckets"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("master_key_id")).string().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("master_key_secret_encrypted"))
                            .binary()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("buckets"))
                    .drop_column(Alias::new("master_key_secret_encrypted"))
                    .drop_column(Alias::new("master_key_id"))
                    .to_owned(),
            )
            .await
    }
}
