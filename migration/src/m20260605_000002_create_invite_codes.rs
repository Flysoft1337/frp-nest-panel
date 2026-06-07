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
                    .table(InviteCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InviteCodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InviteCodes::Code)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(InviteCodes::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(InviteCodes::UsedBy).uuid())
                    .col(ColumnDef::new(InviteCodes::UsedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(InviteCodes::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(InviteCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_codes_created_by")
                            .from(InviteCodes::Table, InviteCodes::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_codes_used_by")
                            .from(InviteCodes::Table, InviteCodes::UsedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InviteCodes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum InviteCodes {
    Table,
    Id,
    Code,
    CreatedBy,
    UsedBy,
    UsedAt,
    ExpiresAt,
    CreatedAt,
}
