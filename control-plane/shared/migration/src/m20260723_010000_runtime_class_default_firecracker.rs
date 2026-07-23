use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "UPDATE workloads SET runtime_class = 'firecracker' WHERE runtime_class = 'docker'",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("workloads"))
                    .modify_column(
                        ColumnDef::new(Alias::new("runtime_class"))
                            .string()
                            .not_null()
                            .default("firecracker"),
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
                    .modify_column(
                        ColumnDef::new(Alias::new("runtime_class"))
                            .string()
                            .not_null()
                            .default("docker"),
                    )
                    .to_owned(),
            )
            .await
    }
}
