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
    /// Output directory for schema files
    #[arg(long, env = "EVENT_SCHEMAS_OUT_DIR", default_value = "./schemas")]
    schemas_out_dir: String,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateSchemas(args) => update_schemas::update_schemas(&args.schemas_out_dir),
    }
}
