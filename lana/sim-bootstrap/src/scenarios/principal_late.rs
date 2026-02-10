use std::collections::VecDeque;
use std::time::Duration;

use es_entity::clock::{ClockController, ClockHandle};
use es_entity::prelude::chrono;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditCollectionEvent, CoreCreditEvent, LanaEvent, ObligationType};
use rust_decimal_macros::dec;
use tracing::{event, instrument};

use crate::helpers;

const ONE_DAY: Duration = Duration::from_secs(86400);
const EVENT_WAIT_TIMEOUT: Duration = Duration::from_millis(100);
const ONE_MONTH_DAYS: i64 = 30;

#[instrument(
    name = "sim_bootstrap.principal_late_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn principal_late_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: &ClockHandle,
    clock_ctrl: &ClockController,
) -> anyhow::Result<()> {
    event!(tracing::Level::INFO, "Starting principal late scenario");

    let mut stream = app.outbox().listen_persisted(None);

    let (customer_id, _) = helpers::create_customer(&sub, app, "3-principal-late").await?;
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
                    && *id == proposal_id
                {
                    msg.inject_trace_parent();
                    break;
                }
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalConcluded {
                    id,
                    status: CreditFacilityProposalStatus::Denied,
                })) = &msg.payload
                    && *id == proposal_id
                {
                    anyhow::bail!("Proposal was denied");
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
            }
        }
    }

    let pending_facility = app
        .credit()
        .pending_credit_facilities()
        .find_by_id(&sub, proposal_id)
        .await?
        .expect("pending facility exists");

    app.credit()
        .collaterals()
        .update_collateral_by_id(
            &sub,
            pending_facility.collateral_id,
            Satoshis::try_from_btc(dec!(230))?,
            clock.today(),
        )
        .await?;

    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { id, .. })) = &msg.payload
                    && *id == cf_id
                {
                    msg.inject_trace_parent();
                    break;
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
            }
        }
    }

    let mut obligation_queue: VecDeque<(ObligationType, UsdCents)> = VecDeque::new();
    let mut month_num = 0;
    let mut principal_remaining = UsdCents::ZERO;
    let mut delay_until: Option<chrono::NaiveDate> = None;
    let mut main_loop_done = false;
    let mut principal_payment_date: Option<chrono::NaiveDate> = None;
    let mut principal_paid = false;
    let mut handling_remaining = false;

    let mut facility_completed = false;

    while !facility_completed {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::CreditCollection(CoreCreditCollectionEvent::ObligationDue {
                    entity,
                })) = &msg.payload
                    && CreditFacilityId::from(entity.beneficiary_id) == cf_id
                    && entity.outstanding_amount > UsdCents::ZERO
                {
                    msg.inject_trace_parent();
                    obligation_queue.push_back((entity.obligation_type, entity.outstanding_amount));
                }

                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityCompleted { id, .. })) = &msg.payload
                    && *id == cf_id
                {
                    msg.inject_trace_parent();
                    facility_completed = true;
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
                let current_date = clock.today();

                if !main_loop_done {
                    if let Some(delay_date) = delay_until {
                        if current_date >= delay_date {
                            delay_until = None;
                            if let Some((obligation_type, amount)) = obligation_queue.pop_front() {
                                if obligation_type == ObligationType::Interest {
                                    let _ = app.record_payment_with_date(&sub, cf_id, amount, current_date).await;
                                } else {
                                    principal_remaining += amount;
                                }
                            }
                        }
                    } else if let Some((obligation_type, amount)) = obligation_queue.pop_front() {
                        if month_num < 3 {
                            month_num += 1;
                            delay_until = Some(current_date + chrono::Duration::days(ONE_MONTH_DAYS));
                            obligation_queue.push_front((obligation_type, amount));
                        } else if obligation_type == ObligationType::Interest {
                            let _ = app.record_payment_with_date(&sub, cf_id, amount, current_date).await;
                        } else {
                            principal_remaining += amount;
                        }
                    }

                    if delay_until.is_none() && principal_remaining > UsdCents::ZERO {
                        let facility = app.credit().facilities().find_by_id(&sub, cf_id).await?.unwrap();
                        let total_outstanding = app.credit().outstanding(&facility).await?;
                        if total_outstanding == principal_remaining {
                            main_loop_done = true;
                            principal_payment_date = Some(current_date + chrono::Duration::days(ONE_MONTH_DAYS));
                        }
                    }
                }

                if main_loop_done
                    && !principal_paid
                    && let Some(payment_date) = principal_payment_date
                    && current_date >= payment_date
                {
                    let _ = app.record_payment_with_date(&sub, cf_id, principal_remaining, current_date).await;
                    principal_paid = true;
                    handling_remaining = true;
                }

                if principal_paid && handling_remaining {
                    let has_outstanding = app.credit().facilities().has_outstanding_obligations(&sub, cf_id).await?;
                    if has_outstanding {
                        if let Some((_, amount)) = obligation_queue.pop_front() {
                            let _ = app.record_payment_with_date(&sub, cf_id, amount, current_date).await;
                        }
                    } else {
                        handling_remaining = false;
                        let _ = app.credit().complete_facility(&sub, cf_id).await;
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
        "Principal late scenario completed"
    );

    Ok(())
}
