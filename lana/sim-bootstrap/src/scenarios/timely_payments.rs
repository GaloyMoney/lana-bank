use futures::StreamExt;
use lana_app::{app::LanaApp, credit::CreditFacilityHistoryEntry::*, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use rust_decimal_macros::dec;

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

    let mut n_disbursal = 0;

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
            }
            Some(LanaEvent::Credit(CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                ..
            })) if { cf.id == *id && amount > &UsdCents::ZERO } => {
                app.credit().record_payment(&sub, *id, *amount).await?;
                let facility = app
                    .credit()
                    .find_by_id(&sub, *id)
                    .await?
                    .expect("cf exists");

                n_disbursal += 1;
                if n_disbursal == 3 {
                    let total_outstanding_amount = app.credit().outstanding(&facility).await?;
                    if !total_outstanding_amount.is_zero() {
                        app.credit()
                            .record_payment(&sub, facility.id, total_outstanding_amount)
                            .await?;
                    }
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

    let cf = app
        .credit()
        .find_by_id(&sub, cf.id)
        .await?
        .expect("cf exists");
    assert_eq!(cf.status(), CreditFacilityStatus::Closed);

    let history = app
        .credit()
        .history::<lana_app::credit::CreditFacilityHistoryEntry>(&sub, cf.id)
        .await?;

    let (disbursals_and_interests, repayments) =
        history.iter().fold((0, 0), |(di, p), entry| match entry {
            Disbursal(_) | Interest(_) => (di + 1, p),
            Payment(_) => (di, p + 1),
            _ => (di, p),
        });
    assert_eq!(disbursals_and_interests, 3);
    assert_eq!(repayments, 3);

    Ok(())
}
