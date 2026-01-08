pub use sea_orm_migration::prelude::*;

mod m20251228_110826_create_table;
mod m20260108_051735_update;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251228_110826_create_table::Migration),
            Box::new(m20260108_051735_update::Migration),
        ]
    }
}
