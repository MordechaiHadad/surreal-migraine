use surreal_migraine::types::MigrationRecord;
use surreal_migraine::{MigrationRunner, types::EmbeddedSource};

use surreal_migraine::{Dir, include_dir};
use surrealdb::Surreal;
use surrealdb::engine::local::Mem;

static TEST_MIGRATIONS: Dir = include_dir!("tests/migrations");

#[tokio::test]
async fn test_migrations_apply_successfully() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();

    let source = EmbeddedSource::new(&TEST_MIGRATIONS);

    let runner = MigrationRunner::new(&db, source);
    runner.up().await.unwrap();

    let result: Vec<MigrationRecord> = db.select("migrations").await.unwrap();
    assert_eq!(result.len(), 2, "Should have applied 2 migrations");

    let users: Vec<serde_json::Value> = db
        .query("INFO FOR TABLE users")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert!(!users.is_empty(), "Users table should have been created");
}
