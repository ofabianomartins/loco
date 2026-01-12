use loco_rs::schema::*;
use sea_orm::{ConnectionTrait, Database, DbErr, Statement};
use sea_orm_migration::SchemaManager;
use serial_test::serial;

use loco_rs::rename_table;
use loco_rs::create_table;
use loco_rs::drop_table;
use loco_rs::add_index;
use loco_rs::remove_index;
use loco_rs::add_column;
use loco_rs::remove_column;
use loco_rs::rename_column;

mod posts {

    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
    #[sea_orm(table_name = "cities")]
    pub struct Model {
        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub uuid: Uuid,
        pub title: String,
    }
}

#[tokio::test]
#[serial]
async fn test_create_table() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title));
    })
    .await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;
    assert!(result.is_some());

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_drop_table() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title));
    })
    .await?;

    drop_table!(&manager, users).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_add_column() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
    })
    .await?;

    add_column!(&manager, users, string(posts::Column::Title)).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;

    assert!(result
        .unwrap()
        .try_get::<String>("", "sql")
        .unwrap()
        .contains("title"));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_remove_column() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title));
    })
    .await?;

    remove_column!(&manager, users, title).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;

    assert!(!result
        .unwrap()
        .try_get::<String>("", "sql")
        .unwrap()
        .contains("title"));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_rename_column() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title).name("old_title"));
    })
    .await?;

    rename_column!(&manager, users, old_title, new_title).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;

    assert!(result
        .unwrap()
        .try_get::<String>("", "sql")
        .unwrap()
        .contains("new_title"));

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_add_index() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title));
    })
    .await?;

    add_index!(&manager, users, |i: &mut Index| {
        i.name("idx-users-title").col("title");
    })
    .await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx-users-title'",
        ))
        .await?;

    assert!(result.is_some());

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_remove_index() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
        t.col(string(posts::Column::Title));
    })
    .await?;

    add_index!(&manager, users, |i: &mut Index| {
        i.name("idx-users-title").col("title");
    })
    .await?;

    remove_index!(&manager, users, idx_users_title).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx-users-title'",
        ))
        .await?;

    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_rename_table() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    create_table!(&manager, users, |t| {
        t.col(pk_auto(posts::Entity));
    })
    .await?;

    rename_table!(&manager, users, customers).await?;

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name='customers'",
        ))
        .await?;
    assert!(result.is_some());

    let result = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT name FROM sqlite_master WHERE type='table' AND name='users'",
        ))
        .await?;
    assert!(result.is_none());

    Ok(())
}
