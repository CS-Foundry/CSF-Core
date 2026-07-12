use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ResourceGroupVpnPeers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ResourceGroupVpnPeers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ResourceGroupVpnPeers::ResourceGroupId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ResourceGroupVpnPeers::ClientPublicKey)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ResourceGroupVpnPeers::ClientTunnelIp)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ResourceGroupVpnPeers::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ResourceGroupVpnPeers::Table,
                                ResourceGroupVpnPeers::ResourceGroupId,
                            )
                            .to(ResourceGroups::Table, ResourceGroups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ResourceGroupVpnPeers::Table)
                    .col(ResourceGroupVpnPeers::ResourceGroupId)
                    .name("idx_resource_group_vpn_peers_resource_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ResourceGroupVpnPeers::Table)
                    .col(ResourceGroupVpnPeers::ClientTunnelIp)
                    .col(ResourceGroupVpnPeers::ResourceGroupId)
                    .name("idx_resource_group_vpn_peers_tunnel_ip")
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ResourceGroupVpnPeers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ResourceGroupVpnPeers {
    Table,
    Id,
    ResourceGroupId,
    ClientPublicKey,
    ClientTunnelIp,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ResourceGroups {
    Table,
    Id,
}
