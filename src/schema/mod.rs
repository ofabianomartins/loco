pub mod ddl;
pub use ddl::*;

pub mod col;
pub use col::*;

use sea_orm::{
    sea_query::{
        Alias, ColumnDef, Expr, IntoIden, Table, TableAlterStatement,
        TableCreateStatement,
    },
    ConnectionTrait, DbErr
};
pub use sea_orm_migration::schema::*;
use sea_orm_migration::{prelude::Iden, sea_query, SchemaManager};

#[derive(Iden)]
enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

/// Alter table
pub fn alter<T: IntoIden + 'static>(name: T) -> TableAlterStatement {
    Table::alter().table(name).take()
}

/// Wrapping table schema creation.
pub fn table_auto_tz<T>(name: T) -> TableCreateStatement
where
    T: IntoIden + 'static,
{
    timestamps_tz(Table::create().table(name).if_not_exists().take())
}

// these two are just aliases, original types exist in seaorm already.

#[must_use]
pub fn timestamps_tz(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone(GeneralIds::CreatedAt).default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone(GeneralIds::UpdatedAt).default(Expr::current_timestamp()));
    t.take()
}

/// Create a nullable timestamptz column definition.
pub fn timestamptz_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .timestamp_with_time_zone()
        .null()
        .take()
}

/// Create a non-nullable timestamptz column definition.
pub fn timestamptz<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .timestamp_with_time_zone()
        .not_null()
        .take()
}

/// Create a non-nullable enum column definition.
pub fn enum_type<T>(name: T, enum_name: &str) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .enumeration::<Alias, Alias, Vec<Alias>>(Alias::new(enum_name), vec![])
        .not_null()
        .take()
}

/// Create a nullable enum column definition.
pub fn enum_type_null<T>(name: T, enum_name: &str) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .enumeration::<Alias, Alias, Vec<Alias>>(Alias::new(enum_name), vec![])
        .null()
        .take()
}

/// Create a non-nullable enum column definition with default value.
///
/// # Example
/// ```ignore
/// create_table(m, "users", vec![
///     ("status", ColType::EnumWithDefault("status_enum".to_string(), vec!["pending".to_string(), "active".to_string()], "pending".to_string()))
/// ], vec![]).await;
/// ```
pub fn enum_type_with_default<T>(name: T, enum_name: &str, default_value: &str) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .enumeration::<Alias, Alias, Vec<Alias>>(Alias::new(enum_name), vec![])
        .not_null()
        .default(Expr::val(default_value))
        .take()
}

/// Create a nullable enum column definition with default value.
///
/// # Example
/// ```ignore
/// create_table(m, "users", vec![
///     ("status", ColType::EnumNullWithDefault("status_enum".to_string(), vec!["pending".to_string(), "active".to_string()], "pending".to_string()))
/// ], vec![]).await;
/// ```
pub fn enum_type_null_with_default<T>(name: T, enum_name: &str, default_value: &str) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .enumeration::<Alias, Alias, Vec<Alias>>(Alias::new(enum_name), vec![])
        .null()
        .default(Expr::val(default_value))
        .take()
}

/// Check if an enum type already exists in the database
async fn check_enum_exists(m: &SchemaManager<'_>, enum_name: &str) -> Result<bool, DbErr> {
    match m.get_database_backend() {
        sea_orm::DatabaseBackend::Postgres => {
            let query = format!(
                "SELECT EXISTS (
                    SELECT 1 FROM pg_type 
                    WHERE typname = '{enum_name}' 
                    AND typtype = 'e'
                )"
            );

            let result = m
                .get_connection()
                .query_one(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    query,
                ))
                .await?;

            Ok(result.is_some_and(|row| row.try_get::<bool>("", "exists").unwrap_or(false)))
        }
        sea_orm::DatabaseBackend::Sqlite => {
            // SQLite doesn't have native enum types, so we'll always return false
            // to allow creation of enum-like behavior through CHECK constraints
            Ok(false)
        }
        sea_orm::DatabaseBackend::MySql => {
            // MySQL doesn't support enums in the same way, so we'll always return false
            Ok(false)
        }
    }
}
