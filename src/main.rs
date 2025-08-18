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
            if a.temporal {
                let path = fs::create_temporal_migration(&dir, &a.name)?;
                tracing::info!("created {}", path.display());
            } else {
                let path = fs::create_numeric_migration(&dir, &a.name)?;
                tracing::info!("created {}", path.display());
            }
        }
    }

    Ok(())
}
