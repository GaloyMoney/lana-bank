#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;

use futures::StreamExt;
use rust_decimal_macros::dec;

use lana_app::{
    app::LanaApp,
    customer::CustomerType,
    primitives::*,
    terms::{Duration, InterestInterval, TermValues},
};
use lana_events::*;

pub use config::*;

pub async fn run(
    superuser_email: String,
    app: &LanaApp,
    config: BootstrapConfig,
) -> anyhow::Result<()> {
    let sub = superuser_subject(&superuser_email, app).await?;

    let customer_ids = create_customers(&sub, app, &config).await?;

    make_deposit(&sub, app, &customer_ids, &config).await?;

    let mut handles = Vec::new();

    for customer_id in customer_ids {
        for _ in 0..config.num_facilities {
            let spawned_app = app.clone();

            let handle = tokio::spawn(async move {
                create_and_process_facility(sub, customer_id, spawned_app).await
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
    customer_id: CustomerId,
    app: LanaApp,
) -> anyhow::Result<()> {
    let terms = std_terms();

    let mut stream = app.outbox().listen_persisted(None).await?;

    let deposit_account = app
        .deposits()
        .list_accounts_by_created_at_for_account_holder(
            &sub,
            customer_id,
            Default::default(),
            es_entity::ListDirection::Descending,
        )
        .await?
        .entities
        .into_iter()
        .next()
        .expect("Deposit account not found");

    let cf = app
        .credit_facilities()
        .initiate(
            &sub,
            customer_id,
            deposit_account.id,
            UsdCents::try_from_usd(dec!(10_000_000))?,
            terms,
        )
        .await?;

    while let Some(msg) = stream.next().await {
        match &msg.payload {
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityApproved { id })) if cf.id == *id => {
                app.credit_facilities()
                    .update_collateral(&sub, cf.id, Satoshis::try_from_btc(dec!(230))?)
                    .await?;
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { id, .. }))
                if cf.id == *id =>
            {
                app.credit_facilities()
                    .initiate_disbursal(&sub, cf.id, UsdCents::try_from_usd(dec!(1_000_000))?)
                    .await?;
            }
            Some(LanaEvent::Credit(CoreCreditEvent::AccrualExecuted { id, amount, .. }))
                if { cf.id == *id && amount > &UsdCents::ZERO } =>
            {
                let _ = app
                    .credit_facilities()
                    .record_payment(&sub, *id, *amount)
                    .await;
                let mut facility = app
                    .credit_facilities()
                    .find_by_id(&sub, *id)
                    .await?
                    .expect("cf exists");
                if facility.interest_accrual_in_progress().is_none() {
                    app.credit_facilities()
                        .record_payment(&sub, facility.id, facility.total_outstanding().total())
                        .await?;
                    app.credit_facilities()
                        .complete_facility(&sub, facility.id)
                        .await?;
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
) -> anyhow::Result<Vec<CustomerId>> {
    let mut customer_ids = Vec::new();

    for i in 1..=config.num_customers {
        let customer_email = format!("customer{}@example.com", i);
        let telegram = format!("customer{}", i);
        let customer_type = CustomerType::Individual;

        let customer = match app
            .customers()
            .find_by_email(sub, customer_email.clone())
            .await?
        {
            Some(existing_customer) => existing_customer,
            None => {
                app.customers()
                    .create(sub, customer_email.clone(), telegram, customer_type)
                    .await?
            }
        };

        customer_ids.push(customer.id);
    }

    Ok(customer_ids)
}

async fn make_deposit(
    sub: &Subject,
    app: &LanaApp,
    customer_ids: &Vec<CustomerId>,
    config: &BootstrapConfig,
) -> anyhow::Result<()> {
    for customer_id in customer_ids {
        let deposit_account_id = app
            .deposits()
            .list_accounts_by_created_at_for_account_holder(
                sub,
                *customer_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities
            .into_iter()
            .next()
            .expect("Deposit account not found")
            .id;

        let _ = app
            .deposits()
            .record_deposit(
                sub,
                deposit_account_id,
                UsdCents::try_from_usd(
                    rust_decimal::Decimal::from(config.num_facilities) * dec!(10_000_000),
                )?,
                None,
            )
            .await?;
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

fn std_terms() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(Duration::Months(3))
        .incurrence_interval(InterestInterval::EndOfDay)
        .accrual_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .build()
        .unwrap()
}
