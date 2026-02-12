#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

#[cfg(feature = "json-schema")]
mod update_schemas;

#[cfg(not(feature = "json-schema"))]
mod update_schemas {
    pub(crate) fn update_schemas(
        _schemas_out_dir: &str,
        _migrations_out_dir: &str,
        _force_recreate: bool,
    ) -> anyhow::Result<()> {
        println!("json-schema feature is disabled. No schemas to update.");
        Ok(())
    }
}

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
    #[arg(
        long,
        env = "EVENT_SCHEMAS_OUT_DIR",
        default_value = "dev/entity-rollups/schemas"
    )]
    schemas_out_dir: String,

    /// Output directory for migration files
    #[arg(
        long,
        env = "MIGRATIONS_OUT_DIR",
        default_value = "lana/app/migrations"
    )]
    migrations_out_dir: String,

    /// Force recreate by deleting existing schema files first
    #[arg(long)]
    force_recreate: bool,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::UpdateSchemas(args) => update_schemas::update_schemas(
            &args.schemas_out_dir,
            &args.migrations_out_dir,
            args.force_recreate,
        ),
    }
}
