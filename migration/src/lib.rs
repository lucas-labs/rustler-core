use sea_orm_migration::async_trait::async_trait;
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table_market;
mod m20240325_200049_create_table_ticker;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table_market::Migration),
            Box::new(m20240325_200049_create_table_ticker::Migration),
        ]
    }

    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("migrations").into_iden()
    }
}
