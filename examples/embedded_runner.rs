//! Minimal example: embed a `migrations/` directory and inspect embedded migrations.
//!
//! - Embed `migrations/` at compile time with `include_dir!`
//! - Construct an `EmbeddedSource`
//! - List migrations and read the first migration's `up`/optional `down` SQL
use eyre::Result;
use surreal_migraine::Dir;
use surreal_migraine::include_dir;
use surreal_migraine::types::MigrationSource;
use surreal_migraine::types::{EmbeddedSource, MigrationKind};

static MIGRATIONS: Dir = include_dir!("migrations");

fn main() -> Result<()> {
    // Build an EmbeddedSource from the compile-time included directory.
    let src = EmbeddedSource::new(&MIGRATIONS);

    // Discover migrations (returned in discovery order).
    let migrations = src.list()?;
    println!("Found {} migration(s)", migrations.len());

    for m in &migrations {
        let kind = match m.kind {
            MigrationKind::File => "file (up-only)",
            MigrationKind::Paired => "paired (up/down)",
        };
        println!("- {}: {}", m.name, kind);
    }

    // Inspect the first migration's contents (if any).
    if let Some(first) = migrations.get(0) {
        let up = src.get_up(first)?;
        println!(
            "\nFirst migration (`{}`) up.sql length: {} bytes",
            first.name,
            up.len()
        );

        match src.get_down(first)? {
            Some(down) => println!("First migration down.surql length: {} bytes", down.len()),
            None => println!("First migration has no down.surql (up-only)"),
        }
    } else {
        println!("\nNo embedded migrations found.");
        println!(
            "Create a `migrations/` directory at the crate root (compile-time) and add .surql files or paired directories, then recompile."
        );
    }

    Ok(())
}
