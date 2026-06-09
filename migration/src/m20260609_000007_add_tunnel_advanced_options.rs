use sea_orm_migration::prelude::*;

use crate::m20260605_000003_create_tunnels::Tunnels;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tunnels::Table)
                    .add_column(
                        ColumnDef::new(Tunnels::UseEncryption)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Tunnels::UseCompression)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Tunnels::BandwidthLimit)
                            .string_len(32)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Tunnels::BandwidthLimitMode)
                            .string_len(16)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Tunnels::ProxyProtocolVersion)
                            .string_len(8)
                            .null(),
                    )
                    .add_column(ColumnDef::new(Tunnels::Locations).text().null())
                    .add_column(
                        ColumnDef::new(Tunnels::HostHeaderRewrite)
                            .string_len(253)
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
                    .table(Tunnels::Table)
                    .drop_column(Tunnels::HostHeaderRewrite)
                    .drop_column(Tunnels::Locations)
                    .drop_column(Tunnels::ProxyProtocolVersion)
                    .drop_column(Tunnels::BandwidthLimitMode)
                    .drop_column(Tunnels::BandwidthLimit)
                    .drop_column(Tunnels::UseCompression)
                    .drop_column(Tunnels::UseEncryption)
                    .to_owned(),
            )
            .await
    }
}
