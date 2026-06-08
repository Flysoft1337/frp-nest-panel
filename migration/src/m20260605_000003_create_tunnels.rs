use sea_orm_migration::prelude::*;

use crate::m20260605_000001_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tunnels::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tunnels::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tunnels::UserId).uuid().not_null())
                    .col(ColumnDef::new(Tunnels::Name).string_len(64).not_null())
                    .col(ColumnDef::new(Tunnels::Protocol).string_len(16).not_null())
                    .col(
                        ColumnDef::new(Tunnels::LocalHost)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tunnels::LocalPort).integer().not_null())
                    .col(
                        ColumnDef::new(Tunnels::RemotePort)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Tunnels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tunnels_user_id")
                            .from(Tunnels::Table, Tunnels::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tunnels_user_id_name")
                    .table(Tunnels::Table)
                    .col(Tunnels::UserId)
                    .col(Tunnels::Name)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tunnels::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tunnels {
    Table,
    Id,
    UserId,
    Name,
    Protocol,
    LocalHost,
    LocalPort,
    RemotePort,
    CustomDomain,
    TlsMode,
    CertificateId,
    CreatedAt,
}
