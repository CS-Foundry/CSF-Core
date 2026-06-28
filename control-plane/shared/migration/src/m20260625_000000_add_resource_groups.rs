use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ResourceGroups::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ResourceGroups::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ResourceGroups::OrganizationId).uuid().not_null())
                    .col(ColumnDef::new(ResourceGroups::Name).string().not_null())
                    .col(ColumnDef::new(ResourceGroups::Description).string().null())
                    .col(ColumnDef::new(ResourceGroups::InternalCidr).string().not_null())
                    .col(ColumnDef::new(ResourceGroups::Status).string().not_null().default("active"))
                    .col(ColumnDef::new(ResourceGroups::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(ResourceGroups::UpdatedAt).date_time().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ResourceGroups::Table, ResourceGroups::OrganizationId)
                            .to(Organization::Table, Organization::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Workloads::Table)
                    .add_column(ColumnDef::new(Workloads::ResourceGroupId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Volumes::Table)
                    .add_column(ColumnDef::new(Volumes::ResourceGroupId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Networks::Table)
                    .add_column(ColumnDef::new(Networks::ResourceGroupId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ResourceGroups::Table)
                    .col(ResourceGroups::OrganizationId)
                    .name("idx_resource_groups_organization_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Workloads::Table)
                    .col(Workloads::ResourceGroupId)
                    .name("idx_workloads_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Volumes::Table)
                    .col(Volumes::ResourceGroupId)
                    .name("idx_volumes_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Networks::Table)
                    .col(Networks::ResourceGroupId)
                    .name("idx_networks_resource_group_id")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_networks_resource_group_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_volumes_resource_group_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_workloads_resource_group_id").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_resource_groups_organization_id")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Networks::Table)
                    .drop_column(Networks::ResourceGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Volumes::Table)
                    .drop_column(Volumes::ResourceGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Workloads::Table)
                    .drop_column(Workloads::ResourceGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ResourceGroups::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ResourceGroups {
    Table,
    Id,
    OrganizationId,
    Name,
    Description,
    InternalCidr,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organization {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Workloads {
    Table,
    ResourceGroupId,
}

#[derive(DeriveIden)]
enum Volumes {
    Table,
    ResourceGroupId,
}

#[derive(DeriveIden)]
enum Networks {
    Table,
    ResourceGroupId,
}
