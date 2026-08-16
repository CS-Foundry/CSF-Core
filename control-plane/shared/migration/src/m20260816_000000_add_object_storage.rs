use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Buckets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Buckets::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Buckets::Name).string().not_null())
                    .col(ColumnDef::new(Buckets::GarageBucketId).string().null())
                    .col(ColumnDef::new(Buckets::GlobalAlias).string().not_null())
                    .col(
                        ColumnDef::new(Buckets::Exposure)
                            .string()
                            .not_null()
                            .default("internal"),
                    )
                    .col(ColumnDef::new(Buckets::QuotaMaxSize).big_integer().null())
                    .col(ColumnDef::new(Buckets::QuotaMaxObjects).big_integer().null())
                    .col(
                        ColumnDef::new(Buckets::Status)
                            .string()
                            .not_null()
                            .default("provisioning"),
                    )
                    .col(ColumnDef::new(Buckets::OrganizationId).uuid().null())
                    .col(ColumnDef::new(Buckets::ResourceGroupId).uuid().null())
                    .col(ColumnDef::new(Buckets::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Buckets::UpdatedAt).date_time().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Buckets::Table, Buckets::ResourceGroupId)
                            .to(ResourceGroups::Table, ResourceGroups::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BucketAccessKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BucketAccessKeys::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BucketAccessKeys::BucketId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BucketAccessKeys::Name).string().not_null())
                    .col(
                        ColumnDef::new(BucketAccessKeys::GarageKeyId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BucketAccessKeys::Permissions)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BucketAccessKeys::ExpiresAt).date_time().null())
                    .col(
                        ColumnDef::new(BucketAccessKeys::LastRotatedAt)
                            .date_time()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BucketAccessKeys::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(BucketAccessKeys::Table, BucketAccessKeys::BucketId)
                            .to(Buckets::Table, Buckets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GarageNodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GarageNodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GarageNodes::AgentId).uuid().not_null())
                    .col(ColumnDef::new(GarageNodes::GarageNodeId).string().null())
                    .col(ColumnDef::new(GarageNodes::Zone).string().not_null())
                    .col(ColumnDef::new(GarageNodes::CapacityBytes).big_integer().null())
                    .col(
                        ColumnDef::new(GarageNodes::Role)
                            .string()
                            .not_null()
                            .default("storage"),
                    )
                    .col(
                        ColumnDef::new(GarageNodes::Status)
                            .string()
                            .not_null()
                            .default("unknown"),
                    )
                    .col(ColumnDef::new(GarageNodes::LayoutVersion).integer().null())
                    .col(ColumnDef::new(GarageNodes::LastSeenAt).date_time().null())
                    .col(ColumnDef::new(GarageNodes::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(GarageNodes::UpdatedAt).date_time().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(GarageNodes::Table, GarageNodes::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Buckets::Table)
                    .col(Buckets::ResourceGroupId)
                    .name("idx_buckets_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Buckets::Table)
                    .col(Buckets::OrganizationId)
                    .name("idx_buckets_organization_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Buckets::Table)
                    .col(Buckets::GlobalAlias)
                    .name("idx_buckets_global_alias")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(BucketAccessKeys::Table)
                    .col(BucketAccessKeys::BucketId)
                    .name("idx_bucket_access_keys_bucket_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(GarageNodes::Table)
                    .col(GarageNodes::AgentId)
                    .name("idx_garage_nodes_agent_id")
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_garage_nodes_agent_id").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_bucket_access_keys_bucket_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().name("idx_buckets_global_alias").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_buckets_organization_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_buckets_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(GarageNodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BucketAccessKeys::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Buckets::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Buckets {
    Table,
    Id,
    Name,
    GarageBucketId,
    GlobalAlias,
    Exposure,
    QuotaMaxSize,
    QuotaMaxObjects,
    Status,
    OrganizationId,
    ResourceGroupId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BucketAccessKeys {
    Table,
    Id,
    BucketId,
    Name,
    GarageKeyId,
    Permissions,
    ExpiresAt,
    LastRotatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GarageNodes {
    Table,
    Id,
    AgentId,
    GarageNodeId,
    Zone,
    CapacityBytes,
    Role,
    Status,
    LayoutVersion,
    LastSeenAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ResourceGroups {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Agents {
    Table,
    Id,
}
