use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Config::Table).if_exists().to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SystemSettings::Table)
                    .if_not_exists()
                    .col(string(SystemSettings::Key).primary_key())
                    .col(json_binary(SystemSettings::Value))
                    .col(
                        timestamp_with_time_zone(SystemSettings::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO system_settings (key, value, updated_at) \
                 VALUES ('logs.retention_days', '30', now())",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Logs::Table)
                    .if_not_exists()
                    .col(pk_uuid(Logs::Id))
                    .col(string(Logs::Service))
                    .col(string(Logs::Level))
                    .col(string(Logs::Classification))
                    .col(text(Logs::Message))
                    .col(uuid_null(Logs::AgentId))
                    .col(uuid_null(Logs::WorkloadId))
                    .col(uuid_null(Logs::OrganizationId))
                    .col(
                        timestamp_with_time_zone(Logs::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_logs_agent")
                            .from(Logs::Table, Logs::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_logs_workload")
                            .from(Logs::Table, Logs::WorkloadId)
                            .to(Workloads::Table, Workloads::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_logs_organization")
                            .from(Logs::Table, Logs::OrganizationId)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        for (index_name, column) in [
            ("idx_logs_service", Logs::Service),
            ("idx_logs_level", Logs::Level),
            ("idx_logs_classification", Logs::Classification),
            ("idx_logs_agent_id", Logs::AgentId),
            ("idx_logs_workload_id", Logs::WorkloadId),
            ("idx_logs_organization_id", Logs::OrganizationId),
        ] {
            manager
                .create_index(
                    Index::create()
                        .name(index_name)
                        .table(Logs::Table)
                        .col(column)
                        .to_owned(),
                )
                .await?;
        }

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_logs_created_at ON logs (created_at DESC)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Logs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SystemSettings::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Config::Table)
                    .if_not_exists()
                    .col(pk_uuid(Config::Id))
                    .col(json_binary(Config::Config))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Config {
    Table,
    Id,
    #[allow(clippy::enum_variant_names)]
    Config,
}

#[derive(DeriveIden)]
enum SystemSettings {
    Table,
    Key,
    Value,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Logs {
    Table,
    Id,
    Service,
    Level,
    Classification,
    Message,
    AgentId,
    WorkloadId,
    OrganizationId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Agents {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Workloads {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Organization {
    Table,
    Id,
}
