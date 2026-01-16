use es_entity::clock::ClockHandle;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use obix::out::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use tracing::{Instrument, Span, event, instrument};

use crate::helpers;

// Scenario 1: A credit facility that made timely payments and was paid off all according to the initial payment plan
#[tracing::instrument(name = "sim_bootstrap.timely_payments_scenario", skip(app, clock), err)]
pub async fn timely_payments_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: ClockHandle,
) -> anyhow::Result<()> {
    let (customer_id, _) = helpers::create_customer(&sub, app, "1-timely-paid").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

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

    let (tx, rx) = mpsc::channel::<UsdCents>(32);
    let sim_app = app.clone();
    let sim_clock = clock.clone();
    tokio::spawn(
        async move {
            do_timely_payments(sub, sim_app, cf_proposal.id.into(), rx, sim_clock)
                .await
                .expect("timely payments failed");
        }
        .instrument(Span::current()),
    );

    while let Some(msg) = stream.next().await {
        if process_obligation_message(&msg, &cf_proposal, &tx).await? {
            break;
        }
    }

    let cf = app
        .credit()
        .facilities()
        .find_by_id(&sub, cf_proposal.id)
        .await?
        .expect("cf exists");

    let actual_status = cf.status();
    let expected_status = CreditFacilityStatus::Closed;

    if actual_status == expected_status {
        event!(tracing::Level::INFO,
            facility_id = %cf_proposal.id,
            status = ?actual_status,
            "Timely payments scenario completed successfully"
        );
    } else {
        event!(tracing::Level::ERROR,
            facility_id = %cf_proposal.id,
            expected_status = ?expected_status,
            actual_status = ?actual_status,
            "Timely payments scenario failed: unexpected facility status"
        );
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.timely_payments.process_activation_message", skip(message, sub, app, cf_proposal, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
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

#[instrument(name = "sim_bootstrap.timely_payments.process_obligation_message", skip(message, cf_proposal, tx), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_obligation_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
    tx: &mpsc::Sender<UsdCents>,
) -> anyhow::Result<bool> {
    match &message.payload {
        Some(LanaEvent::Credit(
            event @ CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                ..
            },
        )) if { *id == cf_proposal.id.into() && amount > &UsdCents::ZERO } => {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            tx.send(*amount).await?;
        }
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityCompleted { id, .. })) => {
            if *id == cf_proposal.id.into() {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                return Ok(true);
            }
        }
        _ => {}
    }
    Ok(false)
}

#[tracing::instrument(
    name = "sim_bootstrap.timely_payments.do_timely_payments",
    skip(app, obligation_amount_rx, clock),
    err
)]
async fn do_timely_payments(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<UsdCents>,
    clock: ClockHandle,
) -> anyhow::Result<()> {
    while let Some(amount) = obligation_amount_rx.recv().await {
        app.record_payment_with_date(&sub, id, amount, clock.today())
            .await?;

        if !app
            .credit()
            .facilities()
            .has_outstanding_obligations(&sub, id)
            .await?
        {
            break;
        }
    }

    app.credit().complete_facility(&sub, id).await?;

    Ok(())
}
