#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use futures::StreamExt;
use rust_decimal_macros::dec;

use lana_app::{
    app::LanaApp,
    primitives::{CreditFacilityId, Satoshis, Subject, UsdCents},
    terms::{Duration, InterestInterval, TermValues},
};
use lana_events::*;

pub async fn run(superuser_email: String, app: &LanaApp) -> anyhow::Result<()> {
    let sub = superuser_subject(&superuser_email, app).await?;
    initial_setup(&sub, app).await?;
    let id = bootstrap_credit_facility(&sub, app).await?;

    let spawned_app = app.clone();
    let _handle = tokio::spawn(async move {
        let _ = process_repayment(sub, id, spawned_app).await;
    });

    println!("waiting for real time");
    sim_time::wait_until_realtime().await;
    println!("done");
    Ok(())
}

async fn process_repayment(sub: Subject, id: CreditFacilityId, app: LanaApp) -> anyhow::Result<()> {
    let mut stream = app.outbox().listen_persisted(None).await?;

    while let Some(msg) = stream.next().await {
        match &msg.payload {
            Some(LanaEvent::Credit(CreditEvent::AccrualExecuted {
                id: cf_id, amount, ..
            })) if *cf_id == id && amount > &UsdCents::ZERO => {
                let _ = app
                    .credit_facilities()
                    .record_payment(&sub, id, *amount)
                    .await;
                let mut cf = app
                    .credit_facilities()
                    .find_by_id(&sub, id)
                    .await?
                    .expect("cf exists");
                if cf.interest_accrual_in_progress().is_none() {
                    app.credit_facilities()
                        .record_payment(&sub, id, cf.outstanding().total())
                        .await?;
                    app.credit_facilities().complete_facility(&sub, id).await?;
                }
            }
            Some(LanaEvent::Credit(CreditEvent::FacilityCompleted { id: cf_id, .. }))
                if *cf_id == id =>
            {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn initial_setup(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let values = std_terms();
    let _ = app
        .terms_templates()
        .create_terms_template(sub, "bootstrap".to_string(), values)
        .await;

    let _ = app
        .customers()
        .create(
            sub,
            "bootstrap@lana.com".to_string(),
            "bootstrap-telegram".to_string(),
        )
        .await;
    let _ = app
        .customers()
        .create(
            sub,
            "bootstrap-whale@lana.com".to_string(),
            "bootstrap-whale".to_string(),
        )
        .await;
    let customer = app
        .customers()
        .find_by_email(sub, "bootstrap@lana.com".to_string())
        .await?
        .expect("Customer not found");

    let _ = app
        .deposits()
        .record(
            sub,
            customer.id,
            UsdCents::try_from_usd(dec!(1_000_000))?,
            None,
        )
        .await?;
    Ok(())
}

pub async fn superuser_subject(superuser_email: &String, app: &LanaApp) -> anyhow::Result<Subject> {
    let superuser = app
        .users()
        .find_by_email(None, superuser_email)
        .await?
        .expect("Superuser not found");
    Ok(Subject::from(superuser.id))
}

pub async fn bootstrap_credit_facility(
    sub: &Subject,
    app: &LanaApp,
) -> anyhow::Result<CreditFacilityId> {
    let customer_email = "bootstrap@lana.com".to_string();
    let customer = app
        .customers()
        .find_by_email(sub, customer_email)
        .await?
        .expect("Superuser not found");
    let terms = std_terms();
    let cf = app
        .credit_facilities()
        .initiate(
            sub,
            customer.id,
            UsdCents::try_from_usd(dec!(10_000_000))?,
            terms,
        )
        .await?;
    let id = cf.id;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    app.credit_facilities()
        .update_collateral(sub, id, Satoshis::try_from_btc(dec!(230))?)
        .await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    app.credit_facilities()
        .initiate_disbursal(sub, id, UsdCents::try_from_usd(dec!(1_000_000))?)
        .await?;
    Ok(id)
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
        .build()
        .unwrap()
}
