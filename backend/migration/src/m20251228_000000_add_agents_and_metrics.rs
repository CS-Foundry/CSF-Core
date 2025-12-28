use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create agents table
        manager
            .create_table(
                Table::create()
                    .table(Agents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Agents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Agents::Name).string().not_null())
                    .col(ColumnDef::new(Agents::Hostname).string().not_null())
                    .col(ColumnDef::new(Agents::IpAddress).string())
                    .col(ColumnDef::new(Agents::AgentVersion).string().not_null())
                    .col(ColumnDef::new(Agents::OsType).string().not_null())
                    .col(ColumnDef::new(Agents::OsVersion).string().not_null())
                    .col(ColumnDef::new(Agents::Architecture).string().not_null())
                    .col(ColumnDef::new(Agents::Status).string().not_null())
                    .col(ColumnDef::new(Agents::LastHeartbeat).date_time())
                    .col(ColumnDef::new(Agents::RegisteredAt).date_time().not_null())
                    .col(ColumnDef::new(Agents::UpdatedAt).date_time())
                    .col(ColumnDef::new(Agents::OrganizationId).uuid())
                    .col(ColumnDef::new(Agents::Tags).json())
                    .col(ColumnDef::new(Agents::Capabilities).json())
                    .to_owned(),
            )
            .await?;

        // Create agent_metrics table
        manager
            .create_table(
                Table::create()
                    .table(AgentMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentMetrics::AgentId).uuid().not_null())
                    .col(ColumnDef::new(AgentMetrics::Timestamp).date_time().not_null())
                    .col(ColumnDef::new(AgentMetrics::CpuModel).string())
                    .col(ColumnDef::new(AgentMetrics::CpuCores).integer())
                    .col(ColumnDef::new(AgentMetrics::CpuThreads).integer())
                    .col(ColumnDef::new(AgentMetrics::CpuUsagePercent).float())
                    .col(ColumnDef::new(AgentMetrics::MemoryTotalBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::MemoryUsedBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::MemoryUsagePercent).float())
                    .col(ColumnDef::new(AgentMetrics::DiskTotalBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::DiskUsedBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::DiskUsagePercent).float())
                    .col(ColumnDef::new(AgentMetrics::NetworkRxBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::NetworkTxBytes).big_integer())
                    .col(ColumnDef::new(AgentMetrics::OsName).string())
                    .col(ColumnDef::new(AgentMetrics::OsVersion).string())
                    .col(ColumnDef::new(AgentMetrics::KernelVersion).string())
                    .col(ColumnDef::new(AgentMetrics::Hostname).string())
                    .col(ColumnDef::new(AgentMetrics::UptimeSeconds).big_integer())
                    .col(ColumnDef::new(AgentMetrics::CustomMetrics).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_metrics_agent")
                            .from(AgentMetrics::Table, AgentMetrics::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on agent_id and timestamp for faster queries
        manager
            .create_index(
                Index::create()
                    .name("idx_agent_metrics_agent_timestamp")
                    .table(AgentMetrics::Table)
                    .col(AgentMetrics::AgentId)
                    .col(AgentMetrics::Timestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentMetrics::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Agents::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Agents {
    Table,
    Id,
    Name,
    Hostname,
    IpAddress,
    AgentVersion,
    OsType,
    OsVersion,
    Architecture,
    Status,
    LastHeartbeat,
    RegisteredAt,
    UpdatedAt,
    OrganizationId,
    Tags,
    Capabilities,
}

#[derive(DeriveIden)]
enum AgentMetrics {
    Table,
    Id,
    AgentId,
    Timestamp,
    CpuModel,
    CpuCores,
    CpuThreads,
    CpuUsagePercent,
    MemoryTotalBytes,
    MemoryUsedBytes,
    MemoryUsagePercent,
    DiskTotalBytes,
    DiskUsedBytes,
    DiskUsagePercent,
    NetworkRxBytes,
    NetworkTxBytes,
    OsName,
    OsVersion,
    KernelVersion,
    Hostname,
    UptimeSeconds,
    CustomMetrics,
}
