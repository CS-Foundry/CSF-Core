use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WorkloadStacks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(WorkloadStacks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WorkloadStacks::ResourceGroupId).uuid().not_null())
                    .col(ColumnDef::new(WorkloadStacks::Name).string().not_null())
                    .col(ColumnDef::new(WorkloadStacks::ComposeSource).text().null())
                    .col(ColumnDef::new(WorkloadStacks::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(WorkloadStacks::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(WorkloadStacks::UpdatedAt).date_time().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(WorkloadStacks::Table, WorkloadStacks::ResourceGroupId)
                            .to(ResourceGroups::Table, ResourceGroups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Workloads::Table)
                    .add_column(ColumnDef::new(Workloads::StackId).uuid().null())
                    .add_column(ColumnDef::new(Workloads::ServiceName).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_workloads_stack_id")
                    .from(Workloads::Table, Workloads::StackId)
                    .to(WorkloadStacks::Table, WorkloadStacks::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(WorkloadStacks::Table)
                    .col(WorkloadStacks::ResourceGroupId)
                    .name("idx_workload_stacks_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Workloads::Table)
                    .col(Workloads::StackId)
                    .name("idx_workloads_stack_id")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_workloads_stack_id").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_workload_stacks_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_workloads_stack_id")
                    .table(Workloads::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Workloads::Table)
                    .drop_column(Workloads::ServiceName)
                    .drop_column(Workloads::StackId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(WorkloadStacks::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum WorkloadStacks {
    Table,
    Id,
    ResourceGroupId,
    Name,
    ComposeSource,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ResourceGroups {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Workloads {
    Table,
    StackId,
    ServiceName,
}
