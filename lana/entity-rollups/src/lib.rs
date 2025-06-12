#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod update_schemas;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "entity-rollups")]
#[command(about = "A tool for managing entity rollup schemas")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    UpdateSchemas(UpdateSchemasArgs),
}

#[derive(Args)]
struct UpdateSchemasArgs {
    // No additional arguments needed for now
}


pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateSchemas(_) => update_schemas::update_schemas(),
    }
}
