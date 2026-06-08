use sea_orm_migration::prelude::*;

use crate::{m20260605_000001_create_users::Users, m20260605_000003_create_tunnels::Tunnels};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tunnels::Table)
                    .modify_column(ColumnDef::new(Tunnels::RemotePort).integer().null())
                    .add_column(ColumnDef::new(Tunnels::CustomDomain).string_len(253).null())
                    .add_column(ColumnDef::new(Tunnels::TlsMode).string_len(32).null())
                    .add_column(ColumnDef::new(Tunnels::CertificateId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Certificates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Certificates::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Certificates::UserId).uuid().not_null())
                    .col(ColumnDef::new(Certificates::Name).string_len(64).not_null())
                    .col(ColumnDef::new(Certificates::DomainsJson).text().not_null())
                    .col(ColumnDef::new(Certificates::CertPath).text().not_null())
                    .col(ColumnDef::new(Certificates::KeyPath).text().not_null())
                    .col(
                        ColumnDef::new(Certificates::NotBefore)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Certificates::NotAfter)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Certificates::FingerprintSha256)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Certificates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_certificates_user_id")
                            .from(Certificates::Table, Certificates::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_certificates_user_id")
                    .table(Certificates::Table)
                    .col(Certificates::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_tunnels_certificate_id")
                    .from(Tunnels::Table, Tunnels::CertificateId)
                    .to(Certificates::Table, Certificates::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_tunnels_certificate_id")
                    .table(Tunnels::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Certificates::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tunnels::Table)
                    .drop_column(Tunnels::CustomDomain)
                    .drop_column(Tunnels::TlsMode)
                    .drop_column(Tunnels::CertificateId)
                    .modify_column(ColumnDef::new(Tunnels::RemotePort).integer().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum Certificates {
    Table,
    Id,
    UserId,
    Name,
    DomainsJson,
    CertPath,
    KeyPath,
    NotBefore,
    NotAfter,
    FingerprintSha256,
    CreatedAt,
}
