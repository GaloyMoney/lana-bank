use std::time::Duration;

use es_entity::clock::{ClockController, ClockHandle};
use es_entity::prelude::chrono;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent, ObligationType};
use rust_decimal_macros::dec;
use tracing::{event, instrument};

use crate::helpers;

const ONE_DAY: Duration = Duration::from_secs(86400);
const EVENT_WAIT_TIMEOUT: Duration = Duration::from_millis(50);

#[instrument(
    name = "sim_bootstrap.interest_late_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn interest_late_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: ClockHandle,
    clock_ctrl: ClockController,
) -> anyhow::Result<()> {
    event!(tracing::Level::INFO, "Starting interest late scenario");

    let mut stream = app.outbox().listen_persisted(None);

    let (customer_id, _) = helpers::create_customer(&sub, app, "2-interest-late").await?;
    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms();
    let cf_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    let cf_proposal = app
        .create_facility_proposal(&sub, customer_id, cf_amount, cf_terms, None::<CustodianId>)
        .await?;
    let proposal_id = cf_proposal.id;
    let cf_id: CreditFacilityId = proposal_id.into();

    app.credit()
        .proposals()
        .conclude_customer_approval(&sub, proposal_id, true)
        .await?;

    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalConcluded {
                    id,
                    status: CreditFacilityProposalStatus::Approved,
                })) = &msg.payload
                {
                    if *id == proposal_id {
                        msg.inject_trace_parent();
                        break;
                    }
                }
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalConcluded {
                    id,
                    status: CreditFacilityProposalStatus::Denied,
                })) = &msg.payload
                {
                    if *id == proposal_id {
                        anyhow::bail!("Proposal was denied");
                    }
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
            }
        }
    }

    app.credit()
        .update_pending_facility_collateral(
            &sub,
            proposal_id,
            Satoshis::try_from_btc(dec!(230))?,
            clock.today(),
        )
        .await?;

    let activation_date;
    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { id, .. })) = &msg.payload {
                    if *id == cf_id {
                        msg.inject_trace_parent();
                        activation_date = clock.today();
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
            }
        }
    }

    let mut first_interest_amount: Option<UsdCents> = None;
    let mut first_interest_due_date: Option<chrono::NaiveDate> = None;
    let mut first_interest_paid = false;

    let expected_end_date = activation_date + chrono::Duration::days(200);
    let mut facility_completed = false;
    let mut days_past_expected_end = 0;

    while !facility_completed {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::Credit(CoreCreditEvent::ObligationDue {
                    credit_facility_id,
                    amount,
                    obligation_type,
                    ..
                })) = &msg.payload
                {
                    if *credit_facility_id == cf_id && *amount > UsdCents::ZERO {
                        msg.inject_trace_parent();
                        let current_date = clock.today();

                        if *obligation_type == ObligationType::Interest && first_interest_amount.is_none() {
                            first_interest_amount = Some(*amount);
                            first_interest_due_date = Some(current_date);
                        } else {
                            let _ = app.record_payment_with_date(&sub, cf_id, *amount, current_date).await;
                        }
                    }
                }

                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { id, .. })) = &msg.payload {
                    if *id == cf_id {
                        msg.inject_trace_parent();
                        facility_completed = true;
                    }
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
                let current_date = clock.today();

                if !first_interest_paid {
                    if let (Some(amount), Some(due_date)) = (first_interest_amount, first_interest_due_date) {
                        let days_late = (current_date - due_date).num_days();
                        if days_late > 90 {
                            let _ = app.record_payment_with_date(&sub, cf_id, amount, current_date).await;
                            first_interest_paid = true;
                        }
                    }
                }

                if current_date >= expected_end_date {
                    days_past_expected_end += 1;

                    let facility = app.credit().facilities().find_by_id(&sub, cf_id).await?.expect("facility exists");

                    if facility.interest_accrual_cycle_in_progress().is_none() {
                        let total_outstanding = app.credit().outstanding(&facility).await?;

                        if total_outstanding > UsdCents::ZERO {
                            let _ = app.record_payment_with_date(&sub, cf_id, total_outstanding, current_date).await;
                        } else {
                            let _ = app.credit().complete_facility(&sub, cf_id).await;
                        }
                    }

                    if days_past_expected_end > 30 {
                        anyhow::bail!("Facility did not complete within expected timeframe");
                    }
                }
            }
        }
    }

    let cf = app
        .credit()
        .facilities()
        .find_by_id(&sub, cf_id)
        .await?
        .expect("cf exists");
    assert_eq!(cf.status(), CreditFacilityStatus::Closed);

    event!(
        tracing::Level::INFO,
        facility_id = %cf_id,
        "Interest late scenario completed"
    );

    Ok(())
}
