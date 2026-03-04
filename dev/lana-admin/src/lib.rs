#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod cli;
mod client;
mod commands;
mod date;
mod graphql;
mod output;

use clap::Parser;
use serde_json::json;

use cli::{Cli, Command};
use client::auth::AuthClient;
use client::auth::load_saved_login_profile;
use client::{CLI_BUILD_VERSION, GraphQLClient};

pub async fn run_with_cli(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Login {
            admin_url,
            keycloak_url,
            keycloak_client_id,
            username,
            password,
        } => {
            let mut auth = AuthClient::new(
                keycloak_url,
                keycloak_client_id,
                admin_url,
                username.clone(),
                password,
            );
            auth.get_token().await?;
            println!("Login successful");
            Ok(())
        }
        Command::Logout => {
            client::auth::clear_session();
            println!("Session cleared.");
            Ok(())
        }
        Command::Version => {
            let saved = load_saved_login_profile();
            match saved {
                Ok(saved) => {
                    let auth = AuthClient::new(
                        saved.keycloak_url,
                        saved.keycloak_client_id,
                        saved.admin_url.clone(),
                        saved.username,
                        saved.password,
                    );
                    let mut client = GraphQLClient::new(saved.admin_url, auth, cli.verbose);
                    match client.build_info().await {
                        Ok(server) => {
                            if cli.json {
                                output::print_json(&json!({
                                    "cliVersion": CLI_BUILD_VERSION,
                                    "serverVersion": server.version,
                                    "serverBuildProfile": server.build_profile,
                                    "serverBuildTarget": server.build_target,
                                    "serverEnabledFeatures": server.enabled_features,
                                    "versionMatch": server.version == CLI_BUILD_VERSION
                                }))?;
                            } else {
                                output::print_kv(&[
                                    ("CLI Version", CLI_BUILD_VERSION),
                                    ("Server Version", &server.version),
                                    ("Server Build Profile", &server.build_profile),
                                    ("Server Build Target", &server.build_target),
                                    (
                                        "Version Match",
                                        if server.version == CLI_BUILD_VERSION {
                                            "true"
                                        } else {
                                            "false"
                                        },
                                    ),
                                ]);
                                if !server.enabled_features.is_empty() {
                                    println!(
                                        "Server Enabled Features: {}",
                                        server.enabled_features.join(", ")
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            if cli.json {
                                output::print_json(&json!({
                                    "cliVersion": CLI_BUILD_VERSION,
                                    "serverVersion": null,
                                    "versionMatch": null,
                                    "error": format!("Failed to fetch server build info: {e}")
                                }))?;
                            } else {
                                output::print_kv(&[
                                    ("CLI Version", CLI_BUILD_VERSION),
                                    ("Server Version", "unavailable"),
                                ]);
                                eprintln!("WARNING: Failed to fetch server build info: {e}");
                            }
                        }
                    }
                }
                Err(_) => {
                    if cli.json {
                        output::print_json(&json!({
                            "cliVersion": CLI_BUILD_VERSION,
                            "serverVersion": null,
                            "versionMatch": null,
                            "note": "No saved login profile found. Run `lana-admin login` to query server version."
                        }))?;
                    } else {
                        output::print_kv(&[
                            ("CLI Version", CLI_BUILD_VERSION),
                            ("Server Version", "unavailable"),
                        ]);
                        println!(
                            "No saved login profile found. Run `lana-admin login` to query server version."
                        );
                    }
                }
            }
            Ok(())
        }
        command => {
            let saved = load_saved_login_profile()?;
            let auth = AuthClient::new(
                saved.keycloak_url,
                saved.keycloak_client_id,
                saved.admin_url.clone(),
                saved.username,
                saved.password,
            );
            let mut client = GraphQLClient::new(saved.admin_url, auth, cli.verbose);
            match command {
                Command::Prospect { action } => {
                    commands::prospect::execute(&mut client, action, cli.json).await
                }
                Command::Customer { action } => {
                    commands::customer::execute(&mut client, action, cli.json).await
                }
                Command::DepositAccount { action } => {
                    commands::deposit_account::execute(&mut client, action, cli.json).await
                }
                Command::TermsTemplate { action } => {
                    commands::terms_template::execute(&mut client, action, cli.json).await
                }
                Command::CreditFacility { action } => {
                    commands::credit_facility::execute(&mut client, action, cli.json).await
                }
                Command::ApprovalProcess { action } => {
                    commands::approval_process::execute(&mut client, action, cli.json).await
                }
                Command::Collateral { action } => {
                    commands::collateral::execute(&mut client, action, cli.json).await
                }
                Command::Accounting { action } => {
                    commands::accounting::execute(&mut client, action, cli.json).await
                }
                Command::Liquidation { action } => {
                    commands::liquidation::execute(&mut client, action, cli.json).await
                }
                Command::Dashboard { action } => {
                    commands::dashboard::execute(&mut client, action, cli.json).await
                }
                Command::FiscalYear { action } => {
                    commands::fiscal_year::execute(&mut client, action, cli.json).await
                }
                Command::CsvExport { action } => {
                    commands::csv_export::execute(&mut client, action, cli.json).await
                }
                Command::Custodian { action } => {
                    commands::custodian::execute(&mut client, action, cli.json).await
                }
                Command::Document { action } => {
                    commands::document::execute(&mut client, action, cli.json).await
                }
                Command::DomainConfig { action } => {
                    commands::domain_config::execute(&mut client, action, cli.json).await
                }
                Command::Audit { action } => {
                    commands::audit::execute(&mut client, action, cli.json).await
                }
                Command::FinancialStatement { action } => {
                    commands::financial_statement::execute(&mut client, action, cli.json).await
                }
                Command::LoanAgreement { action } => {
                    commands::loan_agreement::execute(&mut client, action, cli.json).await
                }
                Command::User { action } => {
                    commands::user::execute(&mut client, action, cli.json).await
                }
                Command::Role { action } => {
                    commands::role::execute(&mut client, action, cli.json).await
                }
                Command::Report { action } => {
                    commands::report::execute(&mut client, action, cli.json).await
                }
                Command::Withdrawal { action } => {
                    commands::withdrawal::execute(&mut client, action, cli.json).await
                }
                Command::Login { .. } | Command::Logout | Command::Version => unreachable!(),
            }
        }
    }
}

pub async fn run_from_args<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    run_with_cli(cli).await
}
