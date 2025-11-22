use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "orcs")]
#[command(about = "ORCS CLI - Orchestrated Reasoning and Collaboration System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate TypeScript type definitions from Rust schemas
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
}

#[derive(Subcommand)]
enum SchemaAction {
    /// Generate TypeScript types
    Generate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Schema { action } => match action {
            SchemaAction::Generate => commands::schema::generate()?,
        },
    }

    Ok(())
}
