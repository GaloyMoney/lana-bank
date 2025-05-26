use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent, ObligationType};
use rust_decimal_macros::dec;
use tokio::sync::mpsc;

use crate::helpers;

// Scenario 3: A credit facility with an principal payment >90 days late
pub async fn principal_late_scenario(sub: Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (customer_id, deposit_account_id) =
        helpers::create_customer(&sub, app, "3-principal-late").await?;

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

    let (tx, rx) = mpsc::channel::<(ObligationType, UsdCents)>(32);
    let sim_app = app.clone();
    tokio::spawn(async move {
        do_principal_late(sub, sim_app, cf.id, rx)
            .await
            .expect("principal late failed");
    });

    while let Some(msg) = stream.next().await {
        match &msg.payload {
            Some(LanaEvent::Credit(CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                obligation_type,
                ..
            })) if { cf.id == *id && amount > &UsdCents::ZERO } => {
                tx.send((*obligation_type, *amount)).await?;
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
        .credit_facilities()
        .find_by_id(&sub, cf.id)
        .await?
        .expect("cf exists");
    assert_eq!(cf.status(), CreditFacilityStatus::Closed);

    Ok(())
}

async fn do_principal_late(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<(ObligationType, UsdCents)>,
) -> anyhow::Result<()> {
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);
    let mut month_num = 0;
    let mut principal_remaining = UsdCents::ZERO;

    while let Some((obligation_type, amount)) = obligation_amount_rx.recv().await {
        // 3 months of interest payments should be delayed by a month
        if month_num < 3 {
            month_num += 1;
            sim_time::sleep(one_month).await;
        }

        if obligation_type == ObligationType::Interest {
            app.credit()
                .record_payment(&sub, id, amount, sim_time::now().date_naive())
                .await?;
        } else {
            principal_remaining += amount;
        }

        let facility = app
            .credit()
            .credit_facilities()
            .find_by_id(&sub, id)
            .await?
            .unwrap();
        let total_outstanding = app.credit().outstanding(&facility).await?;
        if total_outstanding == principal_remaining {
            break;
        }
    }

    // Delaying payment of principal by one more month
    sim_time::sleep(one_month).await;
    app.credit()
        .record_payment(&sub, id, principal_remaining, sim_time::now().date_naive())
        .await?;

    if app.credit().has_outstanding_obligations(&sub, id).await? {
        while let Some((_, amount)) = obligation_amount_rx.recv().await {
            app.credit()
                .record_payment(&sub, id, amount, sim_time::now().date_naive())
                .await?;

            if !app.credit().has_outstanding_obligations(&sub, id).await? {
                break;
            }
        }
    }

    app.credit().complete_facility(&sub, id).await?;

    Ok(())
}
