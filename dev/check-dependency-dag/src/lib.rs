pub mod authz;
pub mod dependency_dag;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
    name = "dependency-checker",
    about = "Lana Bank - Dependency and Authorization Checker",
    long_about = "A tool to check dependency DAG violations and GraphQL mutation authorization in the Lana Bank codebase",
    after_help = "EXAMPLES:\n    dependency-checker check-authz    # Check GraphQL mutation authorization\n    dependency-checker check-dag      # Check dependency violations"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check that all GraphQL mutations have proper authorization
    ///
    /// This command parses the GraphQL schema.rs file and verifies that every
    /// mutation function calls appropriate authorization methods like:
    /// - authz.enforce_permission()
    /// - subject_can_*() helper methods
    /// - Functions that delegate authorization properly
    CheckAuthz,
    /// Check dependency DAG violations between architectural tiers
    ///
    /// This command enforces the architectural rule that packages can only
    /// depend on packages in the same tier or lower tiers:
    /// - lana/* (application layer) can depend on core/* and lib/*
    /// - core/* (domain layer) can depend on lib/*
    /// - lib/* (infrastructure layer) cannot depend on higher tiers
    CheckDag,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CheckAuthz => {
            println!("🔐 Checking authorization for GraphQL mutations...");
            authz::check_authorization().await
        }
        Commands::CheckDag => {
            println!("🏗️ Checking dependency DAG violations...");
            dependency_dag::check_dependency_dag().await
        }
    }
}
