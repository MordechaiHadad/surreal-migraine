use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "surreal-migrate")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new migration file
    Add(AddArgs),
}

#[derive(clap::Args, Debug)]
pub struct AddArgs {
    /// Name of the migration (will be sanitized)
    pub name: String,

    /// Use temporal (timestamp) prefix instead of numeric
    #[arg(short, long)]
    pub temporal: bool,

    /// Override migrations directory
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
