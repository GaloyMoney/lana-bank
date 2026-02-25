use std::time::Duration;

use es_entity::clock::{ClockController, ClockHandle};
use es_entity::prelude::chrono::{self, Utc};
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use rust_decimal_macros::dec;
use tracing::{event, instrument};

use crate::helpers;

const ONE_DAY: Duration = Duration::from_secs(86400);
const MIN_EVENT_WAIT: Duration = Duration::from_millis(100);
const MAX_EVENT_WAIT: Duration = Duration::from_secs(2);

#[instrument(
    name = "sim_bootstrap.interest_under_payment_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn interest_under_payment_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: &ClockHandle,
    clock_ctrl: &ClockController,
) -> anyhow::Result<()> {
    event!(
        tracing::Level::INFO,
        "Starting interest under payment scenario"
    );

    let target_time = Utc::now() - chrono::Duration::days(60);
    clock_ctrl.reset_to(target_time);

    let (customer_id, _) = helpers::create_customer(&sub, app, "5-interest-under-payment").await?;
    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let mut stream = app.outbox().listen_persisted(None);

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

    let mut days_waiting_for_approval = 0;
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
                days_waiting_for_approval += 1;
                if days_waiting_for_approval > 30 {
                    anyhow::bail!("Proposal approval timed out after 30 days");
                }
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

    let mut days_waiting_for_activation = 0;
    let mut wait = MIN_EVENT_WAIT;
    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                wait = MIN_EVENT_WAIT;
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { entity })) = &msg.payload
                    && entity.id == cf_id
                {
                    msg.inject_trace_parent();
                    break;
                }
            }
            _ = tokio::time::sleep(wait) => {
                clock_ctrl.advance(ONE_DAY).await;
                wait = (wait * 2).min(MAX_EVENT_WAIT);
                days_waiting_for_activation += 1;
                if days_waiting_for_activation > 30 {
                    anyhow::bail!("Facility activation timed out after 30 days");
                }
            }
        }
    }

    event!(
        tracing::Level::INFO,
        facility_id = %cf_id,
        "Interest under payment scenario completed - facility active with no payments"
    );

    Ok(())
}
