// #![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod helpers;
mod scenarios;
mod seed;

use std::collections::HashSet;

use futures::StreamExt;
use rust_decimal_macros::dec;

use lana_app::{app::LanaApp, primitives::*};
use lana_events::*;

pub use config::*;

pub async fn run(
    superuser_email: String,
    app: &LanaApp,
    config: BootstrapConfig,
) -> anyhow::Result<()> {
    let sub = superuser_subject(&superuser_email, app).await?;

    seed::seed(&sub, app).await?;

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

            let handle = tokio::spawn(async move {
                create_and_process_facility(sub, spawned_app, customer_id, deposit_account_id).await
            });
            handles.push(handle);
        }
    }

    println!("waiting for real time");
    sim_time::wait_until_realtime().await;
    println!("done");

    Ok(())
}

async fn create_and_process_facility(
    sub: Subject,
    app: LanaApp,
    customer_id: CustomerId,
    deposit_account_id: DepositAccountId,
) -> anyhow::Result<()> {
    let terms = helpers::std_terms();

    let mut stream = app.outbox().listen_persisted(None).await?;

    let cf = app
        .credit()
        .initiate(
            &sub,
            customer_id,
            deposit_account_id,
            UsdCents::try_from_usd(dec!(10_000_000))?,
            terms,
        )
        .await?;

    while let Some(msg) = stream.next().await {
        match &msg.payload {
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityApproved { id })) if cf.id == *id => {
                app.credit()
                    .update_collateral(
                        &sub,
                        cf.id,
                        Satoshis::try_from_btc(dec!(230))?,
                        sim_time::now().date_naive(),
                    )
                    .await?;
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { id, .. }))
                if cf.id == *id =>
            {
                app.credit()
                    .initiate_disbursal(&sub, cf.id, UsdCents::try_from_usd(dec!(1_000_000))?)
                    .await?;
            }
            Some(LanaEvent::Credit(CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                ..
            })) if { cf.id == *id && amount > &UsdCents::ZERO } => {
                let _ = app
                    .credit()
                    .record_payment(&sub, *id, *amount, sim_time::now().date_naive())
                    .await;
                let facility = app
                    .credit()
                    .find_by_id(&sub, *id)
                    .await?
                    .expect("cf exists");
                if facility.interest_accrual_cycle_in_progress().is_none() {
                    let total_outstanding_amount = app.credit().outstanding(&facility).await?;
                    app.credit()
                        .record_payment(
                            &sub,
                            facility.id,
                            total_outstanding_amount,
                            sim_time::now().date_naive(),
                        )
                        .await?;
                    app.credit().complete_facility(&sub, facility.id).await?;
                }
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { id, .. })) => {
                if cf.id == *id {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

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

async fn superuser_subject(superuser_email: &String, app: &LanaApp) -> anyhow::Result<Subject> {
    let superuser = app
        .users()
        .find_by_email(None, superuser_email)
        .await?
        .expect("Superuser not found");
    Ok(Subject::from(superuser.id))
}
