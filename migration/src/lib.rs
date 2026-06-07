pub use sea_orm_migration::prelude::*;

mod m20260605_000001_create_users;
mod m20260605_000002_create_invite_codes;
mod m20260605_000003_create_tunnels;
mod m20260608_000004_add_user_max_tunnels;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260605_000001_create_users::Migration),
            Box::new(m20260605_000002_create_invite_codes::Migration),
            Box::new(m20260605_000003_create_tunnels::Migration),
            Box::new(m20260608_000004_add_user_max_tunnels::Migration),
        ]
    }
}
