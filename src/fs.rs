use crate::name::{parse_numeric_prefix, sanitize_name};
use chrono::Local;
use eyre::{Result, eyre};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Detect an existing `migrations` directory or create one.
/// If `dir_override` is Some(path) that path is used (created if needed).
pub fn detect_or_create_migrations_dir(dir_override: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = dir_override {
        if !d.exists() {
            fs::create_dir_all(&d)?;
        }
        tracing::debug!(dir = %d.display(), "using overridden migrations dir");
        return Ok(d);
    }

    let cwd = std::env::current_dir()?;
    if let Some(name) = cwd.file_name().and_then(|s| s.to_str())
        && name.eq_ignore_ascii_case("migrations")
    {
        return Ok(cwd);
    }

    let candidate = cwd.join("migrations");
    if candidate.exists() {
        tracing::debug!(dir = %candidate.display(), "found existing migrations dir");
        return Ok(candidate);
    }
    fs::create_dir_all(&candidate)?;
    tracing::debug!(dir = %candidate.display(), "created migrations dir");
    Ok(candidate)
}

pub fn next_numeric_prefix(dir: &Path) -> Result<u64> {
    let mut max: Option<u64> = None;
    for entry in fs::read_dir(dir)? {
        let e = entry?;
        if let Some(name) = e.file_name().to_str()
            && name.ends_with(".surql")
            && let Some(n) = parse_numeric_prefix(name)
        {
            max = Some(match max {
                Some(m) => m.max(n),
                None => n,
            });
            tracing::trace!(file = name, prefix = n);
        }
    }
    let next = max.map_or(0, |v| v + 1);
    tracing::debug!(next = next, "computed next numeric prefix");
    Ok(next)
}

/// Create a numeric migration file with a unique filename.
/// The filename is generated based on the next numeric prefix and sanitized name.
pub fn create_numeric_migration(dir: &Path, name: &str) -> Result<PathBuf> {
    let sanitized = sanitize_name(name);
    if sanitized.is_empty() {
        return Err(eyre!("sanitized name is empty"));
    }
    let mut n = next_numeric_prefix(dir)?;
    for _ in 0..1000 {
        let filename = format!("{n:03}_{sanitized}.surql");
        let path = dir.join(&filename);
        match File::options().create_new(true).write(true).open(&path) {
            Ok(mut f) => {
                let header = format!(
                    "-- migration: {name}\n-- created: {now}\n",
                    name = name,
                    now = Local::now()
                );
                let _ = f.write_all(header.as_bytes());
                return Ok(path);
            }
            Err(_) => {
                n += 1;
                continue;
            }
        }
    }
    Err(eyre!(
        "failed to create unique numeric migration after retries"
    ))
}

/// Create a migration file prefixed with a timestamp. If a file with the same
/// name exists, append a numeric suffix until a unique filename is found.
pub fn create_temporal_migration(dir: &Path, name: &str) -> Result<PathBuf> {
    let sanitized = sanitize_name(name);
    if sanitized.is_empty() {
        return Err(eyre!("sanitized name is empty"));
    }
    let ts = Local::now().format("%Y%m%d%H%M%S").to_string();
    let mut path = dir.join(format!("{ts}_{sanitized}.surql"));
    let mut suffix = 1;
    while path.exists() {
        path = dir.join(format!("{ts}_{sanitized}_{suffix}.surql"));
        suffix += 1;
    }
    let mut f = File::create(&path)?;
    let header = format!(
        "-- migration: {name}\n-- created: {now}\n",
        name = name,
        now = Local::now()
    );
    let _ = f.write_all(header.as_bytes());
    Ok(path)
}
