#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod cli;
mod client;
mod commands;
mod date;
mod graphql;
mod output;
mod show_query;

use clap::Parser;
use serde_json::json;

use cli::{AuthAction, Cli, Command, CreditCommand, DepositCommand, IamCommand, SystemCommand};
use client::auth::{
    AuthClient, SavedLoginProfile, load_saved_login_profile, load_saved_session_info,
};
use client::{CLI_BUILD_VERSION, GraphQLClient};

fn default_preview_profile() -> SavedLoginProfile {
    SavedLoginProfile {
        admin_url: "https://admin.staging.galoy.io/graphql".to_string(),
        keycloak_url: "https://auth.staging.galoy.io".to_string(),
        keycloak_client_id: "admin-panel".to_string(),
        username: "admin@galoy.io".to_string(),
        password: String::new(),
    }
}

pub async fn run_with_cli(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Auth { action } => match action {
            AuthAction::Login {
                admin_url,
                keycloak_url,
                keycloak_client_id,
                username,
                password,
            } => {
                let defaults = default_preview_profile();
                let saved = load_saved_login_profile().ok();

                let admin_url = admin_url
                    .or_else(|| saved.as_ref().map(|s| s.admin_url.clone()))
                    .unwrap_or(defaults.admin_url);
                let keycloak_url = keycloak_url
                    .or_else(|| saved.as_ref().map(|s| s.keycloak_url.clone()))
                    .unwrap_or(defaults.keycloak_url);
                let keycloak_client_id = keycloak_client_id
                    .or_else(|| saved.as_ref().map(|s| s.keycloak_client_id.clone()))
                    .unwrap_or(defaults.keycloak_client_id);
                let username = username
                    .or_else(|| saved.as_ref().map(|s| s.username.clone()))
                    .unwrap_or(defaults.username);
                let password = password
                    .or_else(|| saved.as_ref().map(|s| s.password.clone()))
                    .unwrap_or(defaults.password);

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
            AuthAction::Logout => {
                client::auth::clear_session();
                println!("Session cleared.");
                Ok(())
            }
            AuthAction::Info => {
                let Ok(info) = load_saved_session_info() else {
                    if cli.json {
                        output::print_json(&json!({
                            "saved": false,
                        }))?;
                    } else {
                        println!(
                            "No saved auth/session profile found. Run `lana-admin auth login` first."
                        );
                    }
                    return Ok(());
                };

                let environment = infer_environment_name(&info.admin_url);
                let expires_at_rfc3339 =
                    chrono::DateTime::from_timestamp(info.expires_at as i64, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| info.expires_at.to_string());

                if cli.json {
                    output::print_json(&json!({
                        "saved": true,
                        "environment": environment,
                        "adminUrl": info.admin_url,
                        "keycloakUrl": info.keycloak_url,
                        "keycloakClientId": info.keycloak_client_id,
                        "username": info.username,
                        "passwordSet": info.password_set,
                        "tokenExpiresAtEpoch": info.expires_at,
                        "tokenExpiresAt": expires_at_rfc3339,
                        "tokenExpired": info.is_expired,
                        "sessionPath": info.session_path,
                    }))?;
                } else {
                    output::print_kv(&[
                        ("Saved", "true"),
                        ("Environment", environment),
                        ("Admin URL", &info.admin_url),
                        ("Keycloak URL", &info.keycloak_url),
                        ("Keycloak Client ID", &info.keycloak_client_id),
                        ("Username", &info.username),
                        (
                            "Password Set",
                            if info.password_set { "true" } else { "false" },
                        ),
                        ("Token Expires At", &expires_at_rfc3339),
                        (
                            "Token Expired",
                            if info.is_expired { "true" } else { "false" },
                        ),
                        ("Session Path", &info.session_path.display().to_string()),
                    ]);
                }
                Ok(())
            }
        },
        Command::Version => {
            if cli.preview_graphql {
                let saved =
                    load_saved_login_profile().unwrap_or_else(|_| default_preview_profile());
                let auth = AuthClient::new(
                    saved.keycloak_url,
                    saved.keycloak_client_id,
                    saved.admin_url.clone(),
                    saved.username,
                    saved.password,
                );
                let mut client = GraphQLClient::new(saved.admin_url, auth, cli.verbose, true);
                return match client.build_info().await {
                    Err(err) if client::is_preview_complete(&err) => Ok(()),
                    Err(err) => Err(err),
                    Ok(_) => Ok(()),
                };
            }

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
                    let mut client = GraphQLClient::new(saved.admin_url, auth, cli.verbose, false);
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
                            "note": "No saved login profile found. Run `lana-admin auth login` to query server version."
                        }))?;
                    } else {
                        output::print_kv(&[
                            ("CLI Version", CLI_BUILD_VERSION),
                            ("Server Version", "unavailable"),
                        ]);
                        println!(
                            "No saved login profile found. Run `lana-admin auth login` to query server version."
                        );
                    }
                }
            }
            Ok(())
        }
        command => {
            let saved = if cli.preview_graphql {
                load_saved_login_profile().unwrap_or_else(|_| default_preview_profile())
            } else {
                load_saved_login_profile()?
            };

            let auth = AuthClient::new(
                saved.keycloak_url,
                saved.keycloak_client_id,
                saved.admin_url.clone(),
                saved.username,
                saved.password,
            );
            let mut client =
                GraphQLClient::new(saved.admin_url, auth, cli.verbose, cli.preview_graphql);

            let result = match command {
                Command::Prospect { action } => {
                    commands::prospect::execute(&mut client, action, cli.json).await
                }
                Command::Customer { action } => {
                    commands::customer::execute(&mut client, action, cli.json).await
                }
                Command::Deposit { action } => match action {
                    DepositCommand::Account { action } => {
                        commands::deposit_account::execute(&mut client, action, cli.json).await
                    }
                    DepositCommand::Record {
                        deposit_account_id,
                        amount,
                    } => {
                        commands::deposit_account::record_deposit(
                            &mut client,
                            deposit_account_id,
                            amount,
                            cli.json,
                        )
                        .await
                    }
                    DepositCommand::Withdrawal { action } => {
                        commands::withdrawal::execute(&mut client, action, cli.json).await
                    }
                },
                Command::Credit { action } => match action {
                    CreditCommand::TermsTemplate { action } => {
                        commands::terms_template::execute(&mut client, action, cli.json).await
                    }
                    CreditCommand::Facility { action } => {
                        commands::credit_facility::execute(&mut client, action, cli.json).await
                    }
                    CreditCommand::ApprovalProcess { action } => {
                        commands::approval_process::execute(&mut client, action, cli.json).await
                    }
                    CreditCommand::Collateral { action } => {
                        commands::collateral::execute(&mut client, action, cli.json).await
                    }
                    CreditCommand::Liquidation { action } => {
                        commands::liquidation::execute(&mut client, action, cli.json).await
                    }
                    CreditCommand::LoanAgreement { action } => {
                        commands::loan_agreement::execute(&mut client, action, cli.json).await
                    }
                },
                Command::Dashboard { action } => {
                    commands::dashboard::execute(&mut client, action, cli.json).await
                }
                Command::Accounting { action } => {
                    commands::accounting::execute(&mut client, action, cli.json).await
                }
                Command::Document { action } => {
                    commands::document::execute(&mut client, action, cli.json).await
                }
                Command::Audit { action } => {
                    commands::audit::execute(&mut client, action, cli.json).await
                }
                Command::Report { action } => {
                    commands::report::execute(&mut client, action, cli.json).await
                }
                Command::Iam { action } => match action {
                    IamCommand::User { action } => {
                        commands::user::execute(&mut client, action, cli.json).await
                    }
                    IamCommand::Role { action } => {
                        commands::role::execute(&mut client, action, cli.json).await
                    }
                },
                Command::System { action } => match action {
                    SystemCommand::DomainConfig { action } => {
                        commands::domain_config::execute(&mut client, action, cli.json).await
                    }
                    SystemCommand::Custodian { action } => {
                        commands::custodian::execute(&mut client, action, cli.json).await
                    }
                },
                Command::Auth { .. } | Command::Version => unreachable!(),
            };

            match result {
                Err(err) if cli.preview_graphql && client::is_preview_complete(&err) => Ok(()),
                other => other,
            }
        }
    }
}

fn infer_environment_name(admin_url: &str) -> &'static str {
    match admin_url {
        url if url.contains(".staging.") => "staging",
        url if url.contains(".qa.") => "qa",
        _ => "custom",
    }
}

pub async fn run_from_args<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args_vec: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    if show_query::has_show_query_flag(&args_vec) {
        return show_query::run_show_query(&args_vec);
    }

    let cli = Cli::parse_from(args_vec);
    run_with_cli(cli).await
}
