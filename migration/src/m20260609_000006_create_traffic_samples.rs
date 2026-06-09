use sea_orm_migration::prelude::*;

use crate::m20260605_000003_create_tunnels::Tunnels;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrafficSamples::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TrafficSamples::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TrafficSamples::TunnelId).uuid())
                    .col(
                        ColumnDef::new(TrafficSamples::Protocol)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TrafficSamples::ProxyName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TrafficSamples::TrafficIn)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TrafficSamples::TrafficOut)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TrafficSamples::SampledAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_traffic_samples_tunnel_id")
                            .from(TrafficSamples::Table, TrafficSamples::TunnelId)
                            .to(Tunnels::Table, Tunnels::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_samples_proxy_sampled_at")
                    .table(TrafficSamples::Table)
                    .col(TrafficSamples::Protocol)
                    .col(TrafficSamples::ProxyName)
                    .col(TrafficSamples::SampledAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_traffic_samples_tunnel_sampled_at")
                    .table(TrafficSamples::Table)
                    .col(TrafficSamples::TunnelId)
                    .col(TrafficSamples::SampledAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrafficSamples::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TrafficSamples {
    Table,
    Id,
    TunnelId,
    Protocol,
    ProxyName,
    TrafficIn,
    TrafficOut,
    SampledAt,
}
