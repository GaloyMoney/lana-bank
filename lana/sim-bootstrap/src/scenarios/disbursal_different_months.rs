use std::time::Duration;

use es_entity::clock::{ClockController, ClockHandle};
use es_entity::prelude::chrono;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditCollectionEvent, CoreCreditEvent, LanaEvent};
use rust_decimal_macros::dec;
use tracing::{event, instrument};

use crate::helpers;

const ONE_DAY: Duration = Duration::from_secs(86400);
const MIN_EVENT_WAIT: Duration = Duration::from_millis(100);
const MAX_EVENT_WAIT: Duration = Duration::from_secs(2);
const ONE_MONTH_DAYS: i64 = 30;

#[instrument(
    name = "sim_bootstrap.disbursal_different_months_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn disbursal_different_months_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: &ClockHandle,
    clock_ctrl: &ClockController,
) -> anyhow::Result<()> {
    event!(
        tracing::Level::INFO,
        "Starting disbursal different months scenario"
    );

    let mut stream = app.outbox().listen_persisted(None);

    let (customer_id, _) =
        helpers::create_customer(&sub, app, "4-disbursal-different-months").await?;
    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms_12m();
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

    let mut wait = MIN_EVENT_WAIT;
    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                wait = MIN_EVENT_WAIT;
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalConcluded {
                    entity,
                })) = &msg.payload
                    && entity.status == CreditFacilityProposalStatus::Approved
                    && entity.id == proposal_id
                {
                    msg.inject_trace_parent();
                    break;
                }
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityProposalConcluded {
                    entity,
                })) = &msg.payload
                    && entity.status == CreditFacilityProposalStatus::Denied
                    && entity.id == proposal_id
                {
                    anyhow::bail!("Proposal was denied");
                }
            }
            _ = tokio::time::sleep(wait) => {
                clock_ctrl.advance(ONE_DAY).await;
                wait = (wait * 2).min(MAX_EVENT_WAIT);
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

    let activation_date;
    let mut wait = MIN_EVENT_WAIT;
    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                wait = MIN_EVENT_WAIT;
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { entity })) = &msg.payload
                    && entity.id == cf_id
                {
                    msg.inject_trace_parent();
                    activation_date = clock.today();

                    app.credit()
                        .initiate_disbursal(&sub, cf_id, UsdCents::try_from_usd(dec!(1_000_000))?)
                        .await?;

                    break;
                }
            }
            _ = tokio::time::sleep(wait) => {
                clock_ctrl.advance(ONE_DAY).await;
                wait = (wait * 2).min(MAX_EVENT_WAIT);
            }
        }
    }

    let disbursal_2_date = activation_date + chrono::Duration::days(ONE_MONTH_DAYS);
    let disbursal_3_date = activation_date + chrono::Duration::days(ONE_MONTH_DAYS * 3);
    let mut disbursal_2_done = false;
    let mut disbursal_3_done = false;
    let expected_end_date = activation_date + chrono::Duration::days(380);

    let mut wait = MIN_EVENT_WAIT;
    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                wait = MIN_EVENT_WAIT;
                if let Some(LanaEvent::CreditCollection(CoreCreditCollectionEvent::ObligationDue {
                    entity,
                })) = &msg.payload
                    && CreditFacilityId::from(entity.beneficiary_id) == cf_id
                    && entity.outstanding_amount > UsdCents::ZERO
                {
                    msg.inject_trace_parent();
                    app.record_payment_with_date(&sub, cf_id, entity.outstanding_amount, clock.today()).await?;
                }
            }
            _ = tokio::time::sleep(wait) => {
                let current_date = clock.today();

                if !disbursal_2_done && current_date >= disbursal_2_date {
                    app.credit()
                        .initiate_disbursal(&sub, cf_id, UsdCents::try_from_usd(dec!(2_000_000))?)
                        .await?;
                    disbursal_2_done = true;
                }

                if !disbursal_3_done && current_date >= disbursal_3_date {
                    app.credit()
                        .initiate_disbursal(&sub, cf_id, UsdCents::try_from_usd(dec!(5_000_000))?)
                        .await?;
                    disbursal_3_done = true;
                }

                if current_date >= expected_end_date {
                    break;
                }
                clock_ctrl.advance(ONE_DAY).await;
                wait = (wait * 2).min(MAX_EVENT_WAIT);
            }
        }
    }

    loop {
        let facility = app
            .credit()
            .facilities()
            .find_by_id(&sub, cf_id)
            .await?
            .expect("facility exists");

        if facility.interest_accrual_cycle_in_progress().is_some() {
            tokio::time::sleep(MIN_EVENT_WAIT).await;
            continue;
        }

        let total_outstanding = app.credit().outstanding(&facility).await?;
        if total_outstanding.is_zero() {
            break;
        }

        app.record_payment_with_date(&sub, cf_id, total_outstanding, clock.today())
            .await?;
        tokio::time::sleep(MIN_EVENT_WAIT).await;
    }

    let _facility = app.credit().complete_facility(&sub, cf_id).await?;

    event!(tracing::Level::INFO, facility_id = %cf_id, "Disbursal different months scenario completed");
    Ok(())
}
