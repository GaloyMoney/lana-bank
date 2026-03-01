#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod cli;
mod client;
mod commands;
mod graphql;
mod output;
mod spec;

use clap::Parser;

use cli::{Cli, Command};
use client::GraphQLClient;
use client::auth::AuthClient;

pub async fn run_with_cli(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Login => {
            let mut auth = AuthClient::new(
                cli.keycloak_url,
                cli.keycloak_client_id,
                cli.admin_url,
                cli.username.clone(),
                cli.password,
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
        Command::Spec => {
            spec::print()?;
            Ok(())
        }
        command => {
            let auth = AuthClient::new(
                cli.keycloak_url,
                cli.keycloak_client_id,
                cli.admin_url.clone(),
                cli.username,
                cli.password,
            );
            let mut client = GraphQLClient::new(cli.admin_url, auth);
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
                Command::Sumsub { action } => {
                    commands::sumsub::execute(&mut client, action, cli.json).await
                }
                Command::LoanAgreement { action } => {
                    commands::loan_agreement::execute(&mut client, action, cli.json).await
                }
                Command::User { action } => {
                    commands::user::execute(&mut client, action, cli.json).await
                }
                Command::Report { action } => {
                    commands::report::execute(&mut client, action, cli.json).await
                }
                Command::Withdrawal { action } => {
                    commands::withdrawal::execute(&mut client, action, cli.json).await
                }
                Command::Login | Command::Logout | Command::Spec => unreachable!(),
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
