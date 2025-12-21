use eyre::Result;
use include_dir::{Dir, DirEntry};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug, Clone)]
pub enum MigrationKind {
    File,
    Paired,
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub name: String,
    pub kind: MigrationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub id: RecordId,
    pub name: String,
}

pub trait MigrationSource {
    fn list(&self) -> Result<Vec<Migration>>;
    fn get_up(&self, migration: &Migration) -> Result<String>;
    fn get_down(&self, migration: &Migration) -> Result<Option<String>>;
}

pub struct DiskSource {
    source: PathBuf,
}

impl DiskSource {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            source: path.into(),
        }
    }
}

impl MigrationSource for DiskSource {
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

            if !name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
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

pub struct EmbeddedSource<'a> {
    source: &'a Dir<'a>,
}

impl<'a> EmbeddedSource<'a> {
    pub fn new(source: &'a Dir<'a>) -> Self {
        Self { source }
    }
}

impl MigrationSource for EmbeddedSource<'_> {
    fn list(&self) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();

        for entry in self.source.entries() {
            let path = entry.path();

            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if !name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
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

    fn get_up(&self, migration: &Migration) -> Result<String> {
        match migration.kind {
            MigrationKind::Paired => {
                let dir = self
                    .source
                    .get_dir(&migration.name)
                    .ok_or_else(|| eyre::eyre!("migration directory not found"))?;
                let file = dir
                    .get_file("up.surql")
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
