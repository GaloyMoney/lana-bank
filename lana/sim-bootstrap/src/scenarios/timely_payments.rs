use futures::StreamExt;
use lana_app::{
    app::LanaApp,
    credit::{self, CreditFacilityHistoryEntry::*},
    primitives::*,
};
use lana_events::{CoreCreditEvent, LanaEvent};
use rust_decimal_macros::dec;
use tokio::sync::mpsc;

use crate::helpers;

// Scenario 1: A credit facility that made timely payments and was paid off all according to the initial payment plan
pub async fn timely_payments_scenario(sub: Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (customer_id, deposit_account_id) =
        helpers::create_customer(&sub, app, "1-timely-paid").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms();
    let cf_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    let cf = app
        .credit()
        .initiate(&sub, customer_id, deposit_account_id, cf_amount, cf_terms)
        .await?;

    let mut stream = app.outbox().listen_persisted(None).await?;
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

                break;
            }
            _ => {}
        }
    }

    let (tx, rx) = mpsc::channel::<UsdCents>(32);
    let sim_app = app.clone();
    tokio::spawn(async move {
        do_timely_payments(sub, sim_app, cf.id, rx)
            .await
            .expect("timely payments failed");
    });

    while let Some(msg) = stream.next().await {
        match &msg.payload {
            Some(LanaEvent::Credit(CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                ..
            })) if { cf.id == *id && amount > &UsdCents::ZERO } => {
                tx.send(*amount).await?;
            }
            Some(LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { id, .. })) => {
                if cf.id == *id {
                    break;
                }
            }
            _ => {}
        }
    }

    let cf = app
        .credit()
        .find_by_id(&sub, cf.id)
        .await?
        .expect("cf exists");
    assert_eq!(cf.status(), CreditFacilityStatus::Closed);

    let history = app
        .credit()
        .history::<credit::CreditFacilityHistoryEntry>(&sub, cf.id)
        .await?;

    let (disbursals_and_interests, repayments) =
        history.iter().fold((0, 0), |(di, p), entry| match entry {
            Disbursal(_) | Interest(_) => (di + 1, p),
            Payment(_) => (di, p + 1),
            _ => (di, p),
        });
    assert_eq!(disbursals_and_interests, 6);
    assert_eq!(repayments, 6);

    Ok(())
}

async fn do_timely_payments(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<UsdCents>,
) -> anyhow::Result<()> {
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);

    for _ in 0..3 {
        sim_time::sleep(one_month).await;

        let amount = obligation_amount_rx
            .recv()
            .await
            .expect("obligation not received");

        app.credit()
            .record_payment(&sub, id, amount, sim_time::now().date_naive())
            .await?;
    }

    let facility = app.credit().find_by_id(&sub, id).await?.unwrap();
    let total_outstanding = app.credit().outstanding(&facility).await?;
    if !total_outstanding.is_zero() {
        app.credit()
            .record_payment(
                &sub,
                facility.id,
                total_outstanding,
                sim_time::now().date_naive(),
            )
            .await?;
    }

    const MAX_RETRIES: usize = 15;
    for attempt in 0..MAX_RETRIES {
        match app.credit().complete_facility(&sub, facility.id).await {
            Ok(_) => {
                break;
            }
            Err(_) if attempt + 1 < MAX_RETRIES => {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            Err(e) => {
                panic!("Failed to complete facility: {:?}", e);
            }
        }
    }

    Ok(())
}
