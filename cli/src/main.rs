mod cli;
mod consts;
mod fs;
mod name;

use clap::Parser;
use cli::{Args, Commands};
use eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let verbose = match &args.command {
        Commands::Add(a) => a.verbose,
    };

    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        let level = match verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        };
        tracing_subscriber::EnvFilter::new(level)
    };

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    match args.command {
        Commands::Add(a) => {
            let dir = fs::detect_or_create_migrations_dir(a.dir)?;
            // Paired folder (with up/down) is the default. Use --single to
            // create a single .surql file instead, preserving temporal or numeric mode.
            if a.single {
                if a.temporal {
                    let path = fs::create_temporal_migration(&dir, &a.name)?;
                    tracing::info!("created {}", path.display());
                } else {
                    let path = fs::create_numeric_migration(&dir, &a.name)?;
                    tracing::info!("created {}", path.display());
                }
            } else {
                let path = if a.temporal {
                    fs::create_temporal_paired_migration(&dir, &a.name)?
                } else {
                    fs::create_numeric_paired_migration(&dir, &a.name)?
                };
                tracing::info!("created paired migration {}", path.display());
            }
        }
    }

    Ok(())
}
