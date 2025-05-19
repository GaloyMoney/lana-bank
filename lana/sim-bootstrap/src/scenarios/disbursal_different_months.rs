use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use rust_decimal_macros::dec;
use tokio::sync::mpsc;

use crate::helpers;

// Scenario 4: A credit facility that has multiple disbursals making timely payments
pub async fn disbursal_different_months_scenario(
    sub: Subject,
    app: &LanaApp,
) -> anyhow::Result<()> {
    let (customer_id, deposit_account_id) =
        helpers::create_customer(&sub, app, "4-disbursal-different-months").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms_12m();
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

    let sim_app = app.clone();
    tokio::spawn(async move {
        do_disbursal_in_different_months(sub, sim_app, cf.id)
            .await
            .expect("disbursal different months failed");
    });

    let (tx, rx) = mpsc::channel::<UsdCents>(32);
    let sim_app = app.clone();
    tokio::spawn(async move {
        do_timely_payments(sub, sim_app, cf.id, rx)
            .await
            .expect("disbursal different months timely payments failed");
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

    Ok(())
}

async fn do_disbursal_in_different_months(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
) -> anyhow::Result<()> {
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);

    // there is already one disbursal in month 1
    sim_time::sleep(one_month).await;

    // disbursal in month 2
    app.credit()
        .initiate_disbursal(&sub, id, UsdCents::try_from_usd(dec!(2_000_000))?)
        .await?;

    sim_time::sleep(one_month * 3).await;

    // disbursal in month 5
    app.credit()
        .initiate_disbursal(&sub, id, UsdCents::try_from_usd(dec!(5_000_000))?)
        .await?;

    Ok(())
}

async fn do_timely_payments(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<UsdCents>,
) -> anyhow::Result<()> {
    while let Some(amount) = obligation_amount_rx.recv().await {
        app.credit()
            .record_payment(&sub, id, amount, sim_time::now().date_naive())
            .await?;

        let facility = app.credit().find_by_id(&sub, id).await?.unwrap();
        let total_outstanding = app.credit().outstanding(&facility).await?;
        if total_outstanding.is_zero() {
            break;
        }
    }

    app.credit().complete_facility(&sub, id).await?;

    Ok(())
}
