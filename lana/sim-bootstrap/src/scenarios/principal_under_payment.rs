use es_entity::clock::ClockHandle;
use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent, ObligationType};
use obix::out::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use tracing::{Instrument, Span, instrument};

use crate::helpers;

// Scenario 6: A fresh credit facility with interests paid out (principal under payment)
#[tracing::instrument(
    name = "sim_bootstrap.principal_under_payment_scenario",
    skip(app, clock),
    err
)]
pub async fn principal_under_payment_scenario(
    sub: Subject,
    app: &LanaApp,
    clock: ClockHandle,
) -> anyhow::Result<()> {
    let (customer_id, _) = helpers::create_customer(&sub, app, "6-principal-under-payment").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms_with_liquidation();
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

    let (tx, rx) = mpsc::channel::<(ObligationType, UsdCents)>(32);
    let sim_app = app.clone();
    let sim_clock = clock.clone();
    tokio::spawn(
        async move {
            do_principal_under_payment(sub, sim_app, cf_proposal.id.into(), rx, sim_clock)
                .await
                .expect("principal under payment failed");
        }
        .instrument(Span::current()),
    );

    while let Some(msg) = stream.next().await {
        if process_obligation_message(&msg, &cf_proposal, &tx).await? {
            break;
        }
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.principal_under_payment.process_activation_message", skip(message, sub, app, cf_proposal, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
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
        )) if *id == cf_proposal.id => {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            app.credit()
                .update_pending_facility_collateral(
                    sub,
                    *id,
                    Satoshis::try_from_btc(dec!(230))?,
                    clock.now().date_naive(),
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

#[instrument(name = "sim_bootstrap.principal_under_payment.process_obligation_message", skip(message, cf_proposal, tx), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_obligation_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
    tx: &mpsc::Sender<(ObligationType, UsdCents)>,
) -> anyhow::Result<bool> {
    match &message.payload {
        Some(LanaEvent::Credit(
            event @ CoreCreditEvent::ObligationDue {
                credit_facility_id: id,
                amount,
                obligation_type,
                ..
            },
        )) if { *id == cf_proposal.id.into() && amount > &UsdCents::ZERO } => {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            if tx.send((*obligation_type, *amount)).await.is_err() {
                return Ok(true);
            }
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
    name = "sim_bootstrap.do_principal_under_payment",
    skip(app, obligation_amount_rx, clock),
    err
)]
async fn do_principal_under_payment(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<(ObligationType, UsdCents)>,
    clock: ClockHandle,
) -> anyhow::Result<()> {
    let mut principal_remaining = UsdCents::ZERO;

    while let Some((obligation_type, amount)) = obligation_amount_rx.recv().await {
        if obligation_type == ObligationType::Interest {
            app.record_payment_with_date(&sub, id, amount, clock.now().date_naive())
                .await?;
        } else {
            principal_remaining += amount;
        }

        let facility = app
            .credit()
            .facilities()
            .find_by_id(&sub, id)
            .await?
            .unwrap();
        let total_outstanding = app.credit().outstanding(&facility).await?;
        if total_outstanding == principal_remaining {
            break;
        }
    }

    Ok(())
}
