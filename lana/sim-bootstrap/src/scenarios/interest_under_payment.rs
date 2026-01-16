use es_entity::clock::ClockHandle;
use es_entity::prelude::chrono::Utc;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use obix::out::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tracing::{Span, instrument};

use crate::helpers;

// Scenario 5: A fresh credit facility with no previous payments (interest under payment)
#[tracing::instrument(
    name = "sim_bootstrap.interest_under_payment_scenario",
    skip(app, clock),
    err
)]
pub async fn interest_under_payment_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: ClockHandle,
) -> anyhow::Result<()> {
    let (customer_id, _) = helpers::create_customer(&sub, app, "5-interest-under-payment").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    // Wait till 2 months before now
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);
    while clock.now() < Utc::now() - es_entity::prelude::chrono::Duration::days(60) {
        clock.sleep(one_month).await;
    }

    let cf_terms = helpers::std_terms();
    let cf_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    let cf_proposal = app
        .create_facility_proposal(&sub, customer_id, cf_amount, cf_terms, None::<CustodianId>)
        .await?;

    let cf_proposal = app
        .credit()
        .proposals()
        .conclude_customer_approval(&sub, cf_proposal.id, true)
        .await?;

    let mut stream = app.outbox().listen_persisted(None);
    while let Some(msg) = stream.next().await {
        if process_activation_message(&msg, &sub, app, &cf_proposal, &clock).await? {
            break;
        }
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.interest_under_payment.process_activation_message", skip(message, sub, app, cf_proposal, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_activation_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    sub: &Subject,
    app: &LanaApp,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
    clock: &ClockHandle,
) -> anyhow::Result<bool> {
    match &message.payload {
        Some(LanaEvent::Credit(
            event @ CoreCreditEvent::FacilityProposalConcluded {
                id,
                status: CreditFacilityProposalStatus::Approved,
            },
        )) if cf_proposal.id == *id => {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            app.credit()
                .update_pending_facility_collateral(
                    sub,
                    *id,
                    Satoshis::try_from_btc(dec!(230))?,
                    clock.today(),
                )
                .await?;
        }
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityActivated { id, .. }))
            if *id == cf_proposal.id.into() =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}
