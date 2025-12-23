use eyre::Result;
use include_dir::{Dir, DirEntry};
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use surrealdb::RecordId;

/// The kind of migration found in a migration source.
///
/// - `File`: a single `.surql` file containing the "up" migration only.
/// - `Paired`: a directory containing `up.surql` and `down.surql`.
///
/// # Examples
///
/// ```rust
/// use crate::types::MigrationKind;
///
/// let single = MigrationKind::File;
/// let dir = MigrationKind::Paired;
///
/// match single {
///     MigrationKind::File => assert!(true),
///     MigrationKind::Paired => panic!("expected File"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationKind {
    /// A migration stored as a single `.surql` file (up-only).
    File,
    /// A migration stored as a directory with `up.surql` and `down.surql`.
    Paired,
}

/// A migration entry found in a migration source.
///
/// This struct represents a single migration item as discovered by a
/// `MigrationSource`. The `name` is the file name (for `File` migrations)
/// or directory name (for `Paired` migrations). The `kind` indicates how
/// the migration is stored and how the source should load its contents.
///
/// # Examples
///
/// ```rust
/// use crate::types::{Migration, MigrationKind};
///
/// let file_migration = Migration {
///     name: "001_init.surql".to_string(),
///     kind: MigrationKind::File,
/// };
///
/// let paired_migration = Migration {
///     name: "002_add_posts".to_string(),
///     kind: MigrationKind::Paired,
/// };
///
/// assert_eq!(file_migration.kind, MigrationKind::File);
/// assert_eq!(paired_migration.kind, MigrationKind::Paired);
/// ```
#[derive(Debug, Clone)]
pub struct Migration {
    /// The migration's file or directory name (e.g. `001_init.surql` or `002_add_posts`).
    pub name: String,
    /// The storage kind for this migration: `File` or `Paired`.
    pub kind: MigrationKind,
}

/// A persisted record representing an applied migration in the database.
///
/// The `id` field is the SurrealDB-assigned record id for the persisted
/// migration entry. The `name` is the migration identifier (file or
/// directory name) that was applied.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::types::MigrationRecord;
/// use surrealdb::RecordId;
///
/// // `id` is typically returned by SurrealDB when inserting a record.
/// let rec = MigrationRecord {
///     id: /* obtain RecordId from DB */,
///     name: "001_init".to_string(),
/// };
/// println!("applied migration: {}", rec.name);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// The SurrealDB record id assigned to this migration record.
    pub id: RecordId,
    /// The migration's file or directory name.
    pub name: String,
}

/// A source of migrations.
///
/// Implementations of this trait expose migrations from some storage medium
/// (for example, the filesystem or embedded assets) and provide access to the
/// migration contents. Callers should use `list()` to discover available
/// migrations and then `get_up()` / `get_down()` to load their SQL payloads.
///
/// The order of the returned migrations is the order callers should use when
/// applying migrations.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::types::{DiskSource, MigrationSource};
///
/// let src = DiskSource::new("migrations");
/// let migrations = src.list().unwrap();
/// for m in migrations {
///     let up_sql = src.get_up(&m).unwrap();
///     println!("Applying {}: {} bytes", m.name, up_sql.len());
/// }
/// ```
pub trait MigrationSource {
    /// List available migrations.
    ///
    /// Returns a vector of `Migration` entries. The returned order should be
    /// treated as the sequence to apply migrations in.
    fn list(&self) -> Result<Vec<Migration>>;

    /// Load the "up" SQL for the given migration.
    ///
    /// Implementations must return the SQL text used to apply the migration.
    fn get_up(&self, migration: &Migration) -> Result<String>;

    /// Load the "down" SQL for the given migration, if available.
    ///
    /// Returns `Ok(Some(sql))` when a down migration exists, `Ok(None)` when the
    /// migration is up-only, or an `Err` if loading failed.
    fn get_down(&self, migration: &Migration) -> Result<Option<String>>;
}

/// A `MigrationSource` implementation that reads migrations from the filesystem.
///
/// `DiskSource` expects a directory containing migration entries. Each entry
/// may be either a single `.surql` file (treated as `MigrationKind::File`) or
/// a directory (treated as `MigrationKind::Paired`) containing `up.surql` and
/// `down.surql` files. Entries whose names do not start with an ASCII digit
/// are ignored by `list()`.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::types::{DiskSource, MigrationSource};
///
/// // Create a source pointing at the `migrations` directory and list entries.
/// let src = DiskSource::new("migrations");
/// let migrations = src.list().expect("read migrations");
/// for m in migrations {
///     let up = src.get_up(&m).expect("read up");
///     let down = src.get_down(&m).unwrap_or(None);
///     println!("{}: up={} bytes, down={}", m.name, up.len(), down.is_some());
/// }
/// ```
pub struct DiskSource {
    /// Filesystem path to the migrations directory.
    ///
    /// This directory is enumerated by `list()`; files are treated as
    /// `MigrationKind::File` and subdirectories are treated as
    /// `MigrationKind::Paired`.
    source: PathBuf,
}

impl DiskSource {
    /// Create a new `DiskSource` pointing at `path` on the filesystem.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use crate::types::DiskSource;
    ///
    /// // Point the source at a local `migrations` directory.
    /// let src = DiskSource::new("migrations");
    /// let items = src.list().expect("read migrations");
    /// for m in items {
    ///     println!("migration {} (kind={:?})", m.name, m.kind);
    /// }
    /// ```
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            source: path.into(),
        }
    }
}

impl MigrationSource for DiskSource {
    /// Filesystem-backed implementation details.
    ///
    /// - `list()` enumerates directory entries, sorts them, filters out
    ///   entries whose names don't start with an ASCII digit, and maps files
    ///   to `MigrationKind::File` and directories to `MigrationKind::Paired`.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// use crate::types::{DiskSource, MigrationSource};
    /// let src = DiskSource::new("migrations");
    /// let migrations = src.list().expect("read migrations");
    /// for m in migrations {
    ///     println!("found migration: {} (kind={:?})", m.name, m.kind);
    /// }
    /// ```
    fn list(&self) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        let mut entries: Vec<_> = std::fs::read_dir(&self.source)?
            .filter_map(|r| r.ok())
            .collect();

        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();

            let name = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if !name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                continue;
            }

            let kind = if path.is_dir() {
                MigrationKind::Paired
            } else {
                MigrationKind::File
            };

            migrations.push(Migration { name, kind });
        }

        Ok(migrations)
    }

    /// Read the "up" SQL for `migration`.
    ///
    /// For `MigrationKind::Paired` the function reads `<dir>/up.surql`.
    /// For `MigrationKind::File` it reads the file directly.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// use crate::types::{DiskSource, MigrationSource, Migration, MigrationKind};
    /// let src = DiskSource::new("migrations");
    /// let m = Migration { name: "001_init.surql".to_string(), kind: MigrationKind::File };
    /// let up = src.get_up(&m).expect("read up");
    /// println!("up sql: {} bytes", up.len());
    /// ```
    fn get_up(&self, migration: &Migration) -> Result<String> {
        let path = self.source.join(&migration.name);

        match migration.kind {
            MigrationKind::Paired => {
                let up_path = path.join("up.surql");
                let content = read_to_string(up_path)?;
                Ok(content)
            }
            MigrationKind::File => {
                let content = read_to_string(path)?;
                Ok(content)
            }
        }
    }

    /// Read the "down" SQL for `migration`, if present.
    ///
    /// Returns `Ok(Some(sql))` for paired migrations that include `down.surql`,
    /// `Ok(None)` for file-based (up-only) migrations, or an `Err` on IO
    /// failures.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// use crate::types::{DiskSource, MigrationSource, Migration, MigrationKind};
    /// let src = DiskSource::new("migrations");
    /// let m = Migration { name: "002_add_posts".to_string(), kind: MigrationKind::Paired };
    /// match src.get_down(&m).expect("read down") {
    ///     Some(sql) => println!("down sql: {} bytes", sql.len()),
    ///     None => println!("no down migration"),
    /// }
    /// ```
    fn get_down(&self, migration: &Migration) -> Result<Option<String>> {
        let path = self.source.join(&migration.name);

        match migration.kind {
            MigrationKind::Paired => {
                let down_path = path.join("down.surql");
                let content = read_to_string(down_path)?;
                Ok(Some(content))
            }
            MigrationKind::File => Ok(None),
        }
    }
}

/// A `MigrationSource` implementation that reads migrations embedded at
/// compile-time using the `include_dir` crate.
///
/// `EmbeddedSource` wraps an `include_dir::Dir` and exposes the same
/// semantics as `DiskSource`: entries may be either files (mapped to
/// `MigrationKind::File`) or directories (mapped to `MigrationKind::Paired`).
/// Names that do not start with an ASCII digit are ignored by `list()`.
///
/// Use this when you want to embed migration SQL into the binary rather
/// than read from disk at runtime.
///
/// # Examples
///
/// ```rust,ignore
/// use include_dir::include_dir;
/// use crate::types::{EmbeddedSource, MigrationSource};
///
/// // Embed the `migrations` directory at compile time.
/// static MIGS: include_dir::Dir = include_dir!("migrations");
/// let src = EmbeddedSource::new(&MIGS);
/// let migrations = src.list().unwrap();
/// for m in migrations {
///     let up = src.get_up(&m).unwrap();
///     println!("embedded migration {}: {} bytes", m.name, up.len());
/// }
/// ```
pub struct EmbeddedSource<'a> {
    /// Reference to the embedded migration directory provided by
    /// `include_dir`. Contains files and subdirectories representing
    /// migrations (either single-file migrations or paired directories).
    source: &'a Dir<'a>,
}

impl<'a> EmbeddedSource<'a> {
    /// Create a new `EmbeddedSource` from an `include_dir::Dir` reference.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use include_dir::include_dir;
    /// use crate::types::EmbeddedSource;
    ///
    /// // Embed the `migrations` directory at compile time.
    /// static MIGS: include_dir::Dir = include_dir!("migrations");
    /// let src = EmbeddedSource::new(&MIGS);
    /// let migrations = src.list().unwrap();
    /// assert!(!migrations.is_empty());
    /// ```
    pub fn new(source: &'a Dir<'a>) -> Self {
        Self { source }
    }
}

impl MigrationSource for EmbeddedSource<'_> {
    /// List embedded migrations.
    ///
    /// This enumerates entries in the embedded directory, converts names to
    /// UTF-8, filters out entries that don't start with an ASCII digit, and
    /// classifies each entry as `File` or `Paired`.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// let src = EmbeddedSource::new(&MIGS);
    /// let items = src.list().unwrap();
    /// assert!(!items.is_empty());
    /// ```
    fn list(&self) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        for entry in self.source.entries() {
            let path = entry.path();

            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if !name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                continue;
            }

            let kind = match entry {
                DirEntry::File(_) => MigrationKind::File,
                DirEntry::Dir(_) => MigrationKind::Paired,
            };

            migrations.push(Migration { name, kind });
        }

        Ok(migrations)
    }

    /// Read the "up" SQL for the given embedded migration.
    ///
    /// For `MigrationKind::Paired` this reads `up.surql` from the embedded
    /// directory. For `MigrationKind::File` it reads the file contents.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// let src = EmbeddedSource::new(&MIGS);
    /// let m = Migration { name: "001_init.surql".to_string(), kind: MigrationKind::File };
    /// let up = src.get_up(&m).unwrap();
    /// println!("embedded up sql length: {}", up.len());
    /// ```
    fn get_up(&self, migration: &Migration) -> Result<String> {
        match migration.kind {
            MigrationKind::Paired => {
                let file_path = Path::new(&migration.name).join("up.surql");

                let dir = self
                    .source
                    .get_dir(&migration.name)
                    .ok_or_else(|| eyre::eyre!("migration directory not found"))?;

                let file = dir
                    .get_file(file_path)
                    .ok_or_else(|| eyre::eyre!("up.surql not found"))?;
                let content = file
                    .contents_utf8()
                    .ok_or_else(|| eyre::eyre!("failed to read contents of up.surql as UTF-8"))?;
                Ok(content.to_string())
            }
            MigrationKind::File => {
                let file = self
                    .source
                    .get_file(&migration.name)
                    .ok_or_else(|| eyre::eyre!("migration file not found"))?;
                let content = file.contents_utf8().ok_or_else(|| {
                    eyre::eyre!("failed to read contents of migration file as UTF-8")
                })?;
                Ok(content.to_string())
            }
        }
    }

    /// Read the "down" SQL for the given embedded migration, if present.
    ///
    /// Returns `Ok(Some(sql))` when `down.surql` exists in an embedded paired
    /// migration, `Ok(None)` for file-based migrations, or an `Err` if the
    /// embedded asset cannot be read as UTF-8.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// let src = EmbeddedSource::new(&MIGS);
    /// let m = Migration { name: "002_add_posts".to_string(), kind: MigrationKind::Paired };
    /// if let Some(down) = src.get_down(&m).unwrap() {
    ///     println!("embedded down sql: {} bytes", down.len());
    /// }
    /// ```
    fn get_down(&self, migration: &Migration) -> Result<Option<String>> {
        match migration.kind {
            MigrationKind::Paired => {
                let dir = self
                    .source
                    .get_dir(&migration.name)
                    .ok_or_else(|| eyre::eyre!("migration directory not found"))?;
                let file = dir
                    .get_file("down.surql")
                    .ok_or_else(|| eyre::eyre!("down.surql not found"))?;
                let content = file
                    .contents_utf8()
                    .ok_or_else(|| eyre::eyre!("failed to read contents of down.surql as UTF-8"))?;
                Ok(Some(content.to_string()))
            }
            MigrationKind::File => Ok(None),
        }
    }
}
