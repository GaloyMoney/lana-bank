use es_entity::prelude::chrono::Utc;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use outbox::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tracing::{Span, instrument};

use crate::helpers;

// Scenario 5: A fresh credit facility with no previous payments (interest under payment)
#[tracing::instrument(name = "sim_bootstrap.interest_under_payment_scenario", skip(app), err)]
pub async fn interest_under_payment_scenario(sub: Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (customer_id, deposit_account_id) =
        helpers::create_customer(&sub, app, "5-interest-under-payment").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    // Wait till 2 months before now
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);
    while sim_time::now() < Utc::now() - one_month * 2 {
        sim_time::sleep(one_month).await;
    }

    let cf_terms = helpers::std_terms();
    let cf_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    let cf_proposal = app
        .credit()
        .create_facility_proposal(
            &sub,
            customer_id,
            deposit_account_id,
            cf_amount,
            cf_terms,
            None::<CustodianId>,
        )
        .await?;

    let mut stream = app.outbox().listen_persisted(None).await?;
    while let Some(msg) = stream.next().await {
        if process_activation_message(&msg, &sub, app, &cf_proposal).await? {
            break;
        }
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.interest_under_payment.process_activation_message", skip(message, sub, app, cf_proposal), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_activation_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    sub: &Subject,
    app: &LanaApp,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
) -> anyhow::Result<bool> {
    match &message.payload {
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityProposalApproved { id, .. }))
            if cf_proposal.id == *id =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            app.credit()
                .update_pending_facility_collateral(
                    sub,
                    *id,
                    Satoshis::try_from_btc(dec!(230))?,
                    sim_time::now().date_naive(),
                )
                .await?;
        }
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityActivated { id, .. }))
            if *id == cf_proposal.id.into() =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            app.credit()
                .initiate_disbursal(sub, *id, UsdCents::try_from_usd(dec!(1_000_000))?)
                .await?;

            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}
