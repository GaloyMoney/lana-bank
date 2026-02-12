use std::time::Duration;

use es_entity::clock::{ClockController, ClockHandle};
use es_entity::prelude::chrono::{self, Utc};
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditCollectionEvent, CoreCreditEvent, LanaEvent, ObligationType};
use rust_decimal_macros::dec;
use tracing::{event, instrument};

use crate::helpers;

const ONE_DAY: Duration = Duration::from_secs(86400);
const EVENT_WAIT_TIMEOUT: Duration = Duration::from_millis(100);

#[instrument(
    name = "sim_bootstrap.principal_under_payment_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub(super) async fn principal_under_payment_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: &ClockHandle,
    clock_ctrl: &ClockController,
) -> anyhow::Result<()> {
    event!(
        tracing::Level::INFO,
        "Starting principal under payment scenario"
    );

    let target_time = Utc::now() - chrono::Duration::days(240);
    clock_ctrl.reset_to(target_time);

    let (customer_id, _) = helpers::create_customer(&sub, app, "6-principal-under-payment").await?;
    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let mut stream = app.outbox().listen_persisted(None);

    let cf_terms = helpers::std_terms_with_liquidation();
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
                days_waiting_for_activation += 1;
                if days_waiting_for_activation > 30 {
                    anyhow::bail!("Facility activation timed out after 30 days");
                }
            }
        }
    }

    let mut principal_remaining = UsdCents::ZERO;
    let mut scenario_done = false;

    while !scenario_done {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::CreditCollection(CoreCreditCollectionEvent::ObligationDue {
                    entity,
                })) = &msg.payload
                    && CreditFacilityId::from(entity.beneficiary_id) == cf_id
                    && entity.outstanding_amount > UsdCents::ZERO
                {
                    msg.inject_trace_parent();

                    if entity.obligation_type == ObligationType::Interest {
                        let _ =
                            app.record_payment_with_date(&sub, cf_id, entity.outstanding_amount, clock.today())
                                .await;
                    } else {
                        principal_remaining += entity.outstanding_amount;
                    }
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;

                if principal_remaining > UsdCents::ZERO {
                    let facility = app.credit().facilities().find_by_id(&sub, cf_id).await?.unwrap();
                    let total_outstanding = app.credit().outstanding(&facility).await?;

                    if total_outstanding == principal_remaining {
                        scenario_done = true;
                    }
                }

            }
        }
    }

    event!(
        tracing::Level::INFO,
        facility_id = %cf_id,
        principal_outstanding = %principal_remaining,
        "Principal under payment scenario completed - facility active with unpaid principal"
    );

    Ok(())
}
