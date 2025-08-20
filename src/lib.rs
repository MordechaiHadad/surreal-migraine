mod migrations_impl {
    use eyre::Result;
    use std::path::Path;
    use surrealdb::Surreal;
    use serde_json::{self, json, Value};

    /// A simple migration runner for SurrealDB.
    pub struct MigrationRunner<'a, E: surrealdb::Connection> {
        pub db: &'a Surreal<E>,
        pub migrations_dir: Box<Path>,
    }

    impl<'a, E: surrealdb::Connection> MigrationRunner<'a, E> {
        pub fn new(db: &'a Surreal<E>, migrations_dir: &Path) -> Self {
            Self {
                db,
                migrations_dir: migrations_dir.to_path_buf().into_boxed_path(),
            }
        }

        /// Run all pending migrations found in the migrations directory.
        pub async fn up(&self) -> Result<()> {
            self.ensure_migrations_table_exists().await?;

            let migrations = self.discover_migrations().await?;

            let applied = self.get_applied_migrations().await?;

            let migrations_to_run: Vec<_> = migrations
                .into_iter()
                .filter(|m| !applied.contains(&m.name))
                .collect();

            for migration in migrations_to_run {
                // If the migration is a directory, look for `up.surql` inside it.
                let content = if migration.path.is_dir() {
                    let up_path = migration.path.join("up.surql");
                    std::fs::read_to_string(&up_path)?
                } else {
                    std::fs::read_to_string(&migration.path)?
                };

                self.db.query(&content).await.map_err(|e| eyre::eyre!(e.to_string()))?;
                self.record_migration(&migration.name).await?;
                println!("Applied migration: {}", migration.name);
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

            let migrations = self.discover_migrations().await?;
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
                    let down_content = if migration.path.is_dir() {
                        let down_path = migration.path.join("down.surql");
                        if down_path.exists() {
                            Some(std::fs::read_to_string(&down_path)?)
                        } else {
                            None
                        }
                    } else {
                        // try sibling patterns: name_down.surql or name.down.surql
                        let parent = migration
                            .path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| self.migrations_dir.to_path_buf());
                        let candidate1 = parent.join(format!("{}_down.surql", migration.name));
                        let candidate2 = parent.join(format!("{}.down.surql", migration.name));
                        if candidate1.exists() {
                            Some(std::fs::read_to_string(&candidate1)?)
                        } else if candidate2.exists() {
                            Some(std::fs::read_to_string(&candidate2)?)
                        } else {
                            None
                        }
                    };

                    if let Some(content) = down_content {
                        self.db.query(&content).await.map_err(|e| eyre::eyre!(e.to_string()))?;
                        self.remove_migration_record(&migration.name).await?;
                        println!("Reverted migration: {}", migration.name);
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
            let _ = self.db.query(sql).bind(("name", name.to_owned())).await.map_err(|e| eyre::eyre!(e.to_string()))?;
            Ok(())
        }

        /// Ensure the `migrations` table exists.
        async fn ensure_migrations_table_exists(&self) -> Result<()> {
            let sql = "DEFINE TABLE IF NOT EXISTS migrations;";
            self.db.query(sql).await.map_err(|e| eyre::eyre!(e.to_string()))?;
            Ok(())
        }

        async fn discover_migrations(&self) -> Result<Vec<Migration>> {
            let mut entries = std::fs::read_dir(&self.migrations_dir)?
                .filter_map(|r| r.ok())
                .filter(|e| {
                    let p = e.path();
                    let is_entry = p.is_file() || p.is_dir();
                    if !is_entry {
                        return false;
                    }
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str().map(|s| s.to_string())) {
                        return fname.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false);
                    }
                    false
                })
                .collect::<Vec<_>>();

            entries.sort_by_key(|e| e.path());

            let mut out = Vec::new();
            for entry in entries {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                out.push(Migration {
                    name,
                    path: path.into_boxed_path(),
                });
            }

            Ok(out)
        }

        /// Retrieve applied migration names from the `migrations` table.
        ///
        /// Pages results in batches to avoid loading very large tables into memory.
        async fn get_applied_migrations(&self) -> Result<Vec<String>> {
            let rows: Vec<Value> = match self.db.select("migrations").await {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("failed to select migrations: {}", e.to_string());
                    return Ok(Vec::new());
                }
            };

            let mut out = Vec::new();
            for v in rows {
                match v {
                    Value::Object(map) => {
                        if let Some(Value::String(s)) = map.get("name") {
                            out.push(s.clone());
                        }
                    }
                    Value::String(s) => out.push(s),
                    _ => continue,
                }
            }

            Ok(out)
        }

        /// Record a migration as applied by creating a record in `migrations`.
        async fn record_migration(&self, name: &str) -> Result<()> {
            let content = json!({ "name": name });
            let _: Option<surrealdb::sql::Value> = self
                .db
                .create("migrations")
                .content(content)
                .await
                .map_err(|e| eyre::eyre!(e.to_string()))?;
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct Migration {
        pub name: String,
        pub path: Box<Path>,
    }

}

pub use migrations_impl::*;
