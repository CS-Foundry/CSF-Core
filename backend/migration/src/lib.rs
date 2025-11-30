pub use sea_orm_migration::prelude::*;

mod m20251026_150747_key;
mod m20251026_150819_invalid_jwt;
mod m20251026_150828_user;
mod m20251026_151103_config;
mod m20251030_192944_expenses;
mod m20251101_200000_subscriptions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251026_150747_key::Migration),
            Box::new(m20251026_150819_invalid_jwt::Migration),
            Box::new(m20251026_150828_user::Migration),
            Box::new(m20251026_151103_config::Migration),
            Box::new(m20251030_192944_expenses::Migration),
            Box::new(m20251101_200000_subscriptions::Migration),
        ]
    }
}
