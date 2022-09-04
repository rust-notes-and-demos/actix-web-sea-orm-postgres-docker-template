pub use sea_orm_migration::prelude::*;

//❗ After creating a new migration file, remove the sample migration below 👇
mod m20220101_000001_create_todo_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            //❗ After creating a new migration file, remove the sample migration below 👇
            Box::new(m20220101_000001_create_todo_table::Migration)
        ]
    }
}
