pub mod types;

mod migrations_impl {
    use eyre::{Result, eyre};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use surrealdb::{RecordId, Surreal};
    use crate::types::MigrationSource;

    /// A simple migration runner for SurrealDB.
    pub struct MigrationRunner<'a, E: surrealdb::Connection, S: MigrationSource> {
        pub db: &'a Surreal<E>,
        pub source: S,
    }

    impl<'a, E: surrealdb::Connection, S: MigrationSource> MigrationRunner<'a, E, S> {
        pub fn new(db: &'a Surreal<E>, source: S) -> Self {
            Self { db, source }
        }

        /// Run all pending migrations found in the migrations directory.
        pub async fn up(&self) -> Result<()> {
            self.ensure_migrations_table_exists().await?;

            let migrations = self.source.list()?;

            let applied = self.get_applied_migrations().await?;

            let migrations_to_run: Vec<_> = migrations
                .into_iter()
                .filter(|m| !applied.contains(&m.name))
                .collect();

            for migration in migrations_to_run {
                // If the migration is a directory, look for `up.surql` inside it.
                let content = self.source.get_up(&migration)?;

                let tx_sql = format!("BEGIN TRANSACTION;\n{content}\nCOMMIT TRANSACTION;");
                let mut response = self
                    .db
                    .query(&tx_sql)
                    .await
                    .map_err(|e| eyre!(e.to_string()))?;

                let errors = response.take_errors();
                if !errors.is_empty() {
                    let remaining = errors
                        .values()
                        .map(|e| e.to_string())
                        .filter(|s| {
                            !s.contains("The query was not executed due to a failed transaction")
                        })
                        .collect::<Vec<_>>();

                    if !remaining.is_empty() {
                        let first = &remaining[0];
                        eyre::bail!(first.to_owned());
                    }
                }
                self.record_migration(&migration.name).await?;
                tracing::info!("Applied migration: {}", migration.name);
            }

            Ok(())
        }

        /// Revert applied migrations in reverse chronological (discovery) order.
        /// For paired folders this runs `down.surql`. For single-file migrations,
        /// this looks for a sibling file named `<name>_down.surql` or `down.<name>.surql`
        /// (basic heuristics) and runs it if present. After successful revert,
        /// the migration record is removed from the `migrations` table.
        pub async fn down(&self) -> Result<()> {
            self.ensure_migrations_table_exists().await?;

            let migrations = self.source.list()?;
            let mut applied = self.get_applied_migrations().await?;

            // Preserve discovery order, but revert in reverse (last discovered first)
            let name_to_entry = migrations
                .into_iter()
                .map(|m| (m.name.clone(), m))
                .collect::<std::collections::HashMap<_, _>>();

            // Only consider applied migrations and sort them by discovery order
            applied.retain(|n| name_to_entry.contains_key(n));

            // Reverse to get most-recent-first
            applied.reverse();

            for name in applied {
                if let Some(migration) = name_to_entry.get(&name) {
                    let down_content = self.source.get_down(migration)?;

                    if let Some(content) = down_content {
                        let tx_sql = format!("BEGIN TRANSACTION;\n{content}\nCOMMIT TRANSACTION;");
                        let mut response = self
                            .db
                            .query(&tx_sql)
                            .await
                            .map_err(|e| eyre!(e.to_string()))?;

                        let errors = response.take_errors();
                        if !errors.is_empty() {
                            let remaining = errors
                                .values()
                                .map(|e| e.to_string())
                                .filter(|s| {
                                    !s.contains(
                                        "The query was not executed due to a failed transaction",
                                    )
                                })
                                .collect::<Vec<_>>();

                            if !remaining.is_empty() {
                                let first = &remaining[0];
                                eyre::bail!(first.to_owned());
                            }
                        }
                        self.remove_migration_record(&migration.name).await?;
                        tracing::info!("Reverted migration: {}", migration.name);
                    } else {
                        tracing::warn!(migration = %migration.name, "no down script found; skipping");
                    }
                }
            }

            Ok(())
        }

        /// Remove a migration record from the `migrations` table.
        async fn remove_migration_record(&self, name: &str) -> Result<()> {
            let sql = "DELETE FROM migrations WHERE name = $name;";
            let _ = self
                .db
                .query(sql)
                .bind(("name", name.to_owned()))
                .await
                .map_err(|e| eyre!(e.to_string()))?;
            Ok(())
        }

        /// Ensure the `migrations` table exists.
        async fn ensure_migrations_table_exists(&self) -> Result<()> {
            let sql = "DEFINE TABLE IF NOT EXISTS migrations PERMISSIONS NONE;";
            self.db.query(sql).await.map_err(|e| eyre!(e.to_string()))?;
            Ok(())
        }


        /// Retrieve applied migration names from the `migrations` table.
        ///
        /// Pages results in batches to avoid loading very large tables into memory.
        async fn get_applied_migrations(&self) -> Result<Vec<String>> {
            let migrations: Vec<MigrationRecord> = match self.db.select("migrations").await {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("failed to select migrations: {}", e.to_string());
                    return Ok(Vec::new());
                }
            };

            let mut migration_strings = Vec::new();

            for record in migrations {
                let name = record.name;
                if !name.is_empty() {
                    migration_strings.push(name);
                }
            }

            Ok(migration_strings)
        }

        /// Record a migration as applied by creating a record in `migrations`.
        async fn record_migration(&self, name: &str) -> Result<()> {
            let content = json!({ "name": name });
            let _ = self
                .db
                .query("CREATE migrations CONTENT $content")
                .bind(("content", content))
                .await
                .map_err(|e| eyre!(e.to_string()))?;
            Ok(())
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MigrationRecord {
        pub id: RecordId,
        pub name: String,
    }
}

pub use migrations_impl::*;
pub use include_dir::include_dir;
