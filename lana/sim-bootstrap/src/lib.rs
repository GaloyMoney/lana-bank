#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod helpers;
mod scenarios;

use futures::future::join_all;
use std::collections::HashSet;

use es_entity::clock::{ClockController, ClockHandle};
use rust_decimal_macros::dec;
use tracing::{Instrument, Span, info, instrument};

use lana_app::{app::LanaApp, deposit::RequireVerifiedCustomerForAccount, primitives::*};

pub use config::*;

#[instrument(name = "sim_bootstrap.run", skip(app, config, clock, clock_ctrl), fields(num_customers = config.num_customers, num_facilities = config.num_facilities), err)]
pub async fn run(
    superuser_email: String,
    app: &LanaApp,
    config: BootstrapConfig,
    clock: ClockHandle,
    clock_ctrl: ClockController,
) -> anyhow::Result<()> {
    if !config.active {
        return Ok(());
    }

    let sub = superuser_subject(&superuser_email, app).await?;

    // Disable KYC requirement to allow creating deposit accounts for unverified customers
    app.exposed_domain_configs()
        .update::<RequireVerifiedCustomerForAccount>(&sub, false)
        .await?;

    match create_term_templates(&sub, app).await {
        Ok(_) => info!("created term templates"),
        Err(_) => {
            clock_ctrl.transition_to_realtime();
            return Ok(());
        }
    }

    let _ = scenarios::run(&sub, app, clock.clone(), clock_ctrl.clone()).await;

    let mut handles = Vec::new();
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
    for (customer_id, deposit_account_id) in customers {
        for _ in 0..config.num_facilities {
            let app = app.clone();
            let clock = clock.clone();

            let sub = sub.clone();
            let handle = tokio::spawn(
                async move {
                    scenarios::process_facility_lifecycle(
                        sub,
                        app,
                        customer_id,
                        deposit_account_id,
                        clock,
                    )
                    .await
                }
                .instrument(Span::current()),
            );
            handles.push(handle);
        }
    }
    join_all(handles).await;

    info!("transitioning to realtime");
    clock_ctrl.transition_to_realtime();

    Ok(())
}

#[instrument(name = "sim_bootstrap.create_term_templates", skip(sub, app))]
async fn create_term_templates(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let term_values = helpers::std_terms();
    app.terms_templates()
        .create_terms_template(sub, String::from("Lana Bank Terms"), term_values)
        .await?;

    Ok(())
}

#[instrument(name = "sim_bootstrap.create_customers", skip(sub, app, config), fields(num_customers = config.num_customers))]
async fn create_customers(
    sub: &Subject,
    app: &LanaApp,
    config: &BootstrapConfig,
) -> anyhow::Result<HashSet<(CustomerId, DepositAccountId)>> {
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
) -> anyhow::Result<()> {
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
async fn superuser_subject(superuser_email: &String, app: &LanaApp) -> anyhow::Result<Subject> {
    let superuser = app
        .access()
        .users()
        .find_by_email(None, superuser_email)
        .await?
        .expect("Superuser not found");
    Ok(Subject::from(superuser.id))
}
