#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod error;
mod helpers;
mod scenarios;

use std::collections::HashSet;

use lana_app::{app::LanaApp, primitives::*};
use rust_decimal_macros::dec;
use tracing::{Instrument, Span, info, instrument};
use tracing_macros::record_error_severity;

pub use config::*;
use error::SimBootstrapError;

#[record_error_severity]
#[instrument(name = "sim_bootstrap.run", skip(app, config), fields(num_customers = config.num_customers, num_facilities = config.num_facilities), err)]
pub async fn run(
    superuser_email: String,
    app: &LanaApp,
    config: BootstrapConfig,
) -> Result<(), SimBootstrapError> {
    if !config.active {
        return Ok(());
    }

    let sub = superuser_subject(&superuser_email, app).await?;

    create_term_templates(&sub, app).await?;

    // keep the scenarios tokio handles
    let _ = scenarios::run(&sub, app).await?;

    // Bootstrapped test users
    let customers = create_customers(&sub, app, &config).await?;

    make_deposits(
        &sub,
        app,
        &customers
            .iter()
            .map(|(customer_id, _)| *customer_id)
            .collect(),
        &config,
    )
    .await?;

    let mut handles = Vec::new();
    for (customer_id, deposit_account_id) in customers {
        for _ in 0..config.num_facilities {
            let spawned_app = app.clone();

            let handle = tokio::spawn(
                async move {
                    scenarios::process_facility_lifecycle(
                        sub,
                        spawned_app,
                        customer_id,
                        deposit_account_id,
                    )
                    .await
                }
                .instrument(Span::current()),
            );
            handles.push(handle);
        }
    }

    info!("waiting for real time");
    sim_time::wait_until_realtime().await;
    info!("switching to real time");

    Ok(())
}

#[instrument(name = "sim_bootstrap.create_term_templates", skip(sub, app))]
async fn create_term_templates(sub: &Subject, app: &LanaApp) -> Result<(), SimBootstrapError> {
    let term_values = helpers::std_terms();
    app.credit()
        .terms_templates()
        .create_terms_template(sub, String::from("Lana Bank Terms"), term_values)
        .await?;

    Ok(())
}

#[instrument(name = "sim_bootstrap.create_customers", skip(sub, app, config), fields(num_customers = config.num_customers))]
async fn create_customers(
    sub: &Subject,
    app: &LanaApp,
    config: &BootstrapConfig,
) -> Result<HashSet<(CustomerId, DepositAccountId)>, SimBootstrapError> {
    let mut customers = HashSet::new();

    for i in 1..=config.num_customers {
        let (customer_id, deposit_account_id) =
            helpers::create_customer(sub, app, &format!("-sim{i}")).await?;
        customers.insert((customer_id, deposit_account_id));
    }

    Ok(customers)
}

#[instrument(name = "sim_bootstrap.make_deposits", skip(sub, app, customer_ids, config), fields(num_customers = customer_ids.len(), num_facilities = config.num_facilities))]
async fn make_deposits(
    sub: &Subject,
    app: &LanaApp,
    customer_ids: &Vec<CustomerId>,
    config: &BootstrapConfig,
) -> Result<(), SimBootstrapError> {
    let usd_cents = UsdCents::try_from_usd(
        rust_decimal::Decimal::from(config.num_facilities) * dec!(10_000_000),
    )?;

    for customer_id in customer_ids {
        helpers::make_deposit(sub, app, customer_id, usd_cents).await?;
    }

    Ok(())
}

#[instrument(
    name = "sim_bootstrap.superuser_subject",
    skip(app),
    fields(superuser_email)
)]
async fn superuser_subject(
    superuser_email: &String,
    app: &LanaApp,
) -> Result<Subject, SimBootstrapError> {
    let superuser = app
        .access()
        .users()
        .find_by_email(None, superuser_email)
        .await?
        .expect("Superuser not found");
    Ok(Subject::from(superuser.id))
}
