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
const EVENT_WAIT_TIMEOUT: Duration = Duration::from_millis(50);

#[instrument(
    name = "sim_bootstrap.interest_under_payment_scenario",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn interest_under_payment_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: ClockHandle,
    clock_ctrl: ClockController,
) -> anyhow::Result<()> {
    event!(
        tracing::Level::INFO,
        "Starting interest under payment scenario"
    );

    let (customer_id, _) = helpers::create_customer(&sub, app, "5-interest-under-payment").await?;
    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let target_time = Utc::now() - chrono::Duration::days(60);
    let current_time = clock.now();

    if current_time < target_time {
        let duration_to_advance = (target_time - current_time)
            .to_std()
            .unwrap_or(Duration::ZERO);
        clock_ctrl.advance(duration_to_advance).await;
    }

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

    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                if let Some(LanaEvent::Credit(CoreCreditEvent::FacilityActivated { id, .. })) = &msg.payload {
                    if *id == cf_id {
                        msg.inject_trace_parent();
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(EVENT_WAIT_TIMEOUT) => {
                clock_ctrl.advance(ONE_DAY).await;
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
