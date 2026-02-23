mod cli;
mod client;
mod commands;
mod graphql;
mod output;
mod tui;

use clap::Parser;

use cli::{Cli, Command};
use client::GraphQLClient;
use client::auth::AuthClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Login => {
            let mut auth = AuthClient::new(cli.keycloak_url, cli.username.clone(), cli.password);
            auth.get_token().await?;
            println!("Logged in as {}", cli.username);
            Ok(())
        }
        Command::Logout => {
            client::auth::clear_session();
            println!("Session cleared.");
            Ok(())
        }
        command => {
            let auth = AuthClient::new(cli.keycloak_url, cli.username, cli.password);
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
                Command::Tui => tui::run(client).await,
                Command::Login | Command::Logout => unreachable!(),
            }
        }
    }
}
