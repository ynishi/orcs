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
    /// Build ORCS Desktop application
    Build,
    /// Run ORCS Desktop in development mode
    Dev,
    /// Generate TypeScript type definitions from Rust schemas
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    /// Manage project version
    Version {
        #[command(subcommand)]
        action: VersionAction,
    },
}

#[derive(Subcommand)]
enum SchemaAction {
    /// Generate TypeScript types
    Generate,
}

#[derive(Subcommand)]
enum VersionAction {
    /// Bump version across all configuration files
    Bump {
        /// New version number (e.g., 1.0.0)
        version: String,
    },
    /// Show current version
    Show,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => commands::build::run()?,
        Commands::Dev => commands::dev::run()?,
        Commands::Schema { action } => match action {
            SchemaAction::Generate => commands::schema::generate()?,
        },
        Commands::Version { action } => match action {
            VersionAction::Bump { version } => commands::version::bump(&version)?,
            VersionAction::Show => commands::version::show()?,
        },
    }

    Ok(())
}
