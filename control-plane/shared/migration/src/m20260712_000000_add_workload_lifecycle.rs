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
                        ColumnDef::new(Alias::new("desired_state"))
                            .string()
                            .not_null()
                            .default("running"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("restart_requested"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("cpu_usage_percent")).double().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("memory_usage_bytes")).big_integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("network_rx_bytes")).big_integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("network_tx_bytes")).big_integer().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("stats_updated_at")).timestamp().null(),
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
                    .drop_column(Alias::new("desired_state"))
                    .drop_column(Alias::new("restart_requested"))
                    .drop_column(Alias::new("cpu_usage_percent"))
                    .drop_column(Alias::new("memory_usage_bytes"))
                    .drop_column(Alias::new("network_rx_bytes"))
                    .drop_column(Alias::new("network_tx_bytes"))
                    .drop_column(Alias::new("stats_updated_at"))
                    .to_owned(),
            )
            .await
    }
}
