use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent, ObligationType};
use obix::out::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use tracing::{Instrument, Span, instrument};

use crate::helpers;

// Scenario 2: A credit facility with an interest payment >90 days late
#[tracing::instrument(name = "sim_bootstrap.interest_late_scenario", skip(app), err)]
pub async fn interest_late_scenario(sub: Subject, app: &LanaApp) -> anyhow::Result<()> {
    let (customer_id, _) = helpers::create_customer(&sub, app, "2-interest-late").await?;

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
        if process_activation_message(&msg, &sub, app, &cf_proposal).await? {
            break;
        }
    }

    let (tx, rx) = mpsc::channel::<(ObligationType, UsdCents)>(32);
    let sim_app = app.clone();
    tokio::spawn(
        async move {
            do_interest_late(sub, sim_app, cf_proposal.id.into(), rx)
                .await
                .expect("interest late failed");
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
    assert_eq!(cf.status(), CreditFacilityStatus::Closed);

    Ok(())
}

#[instrument(name = "sim_bootstrap.interest_late.process_activation_message", skip(message, sub, app, cf_proposal), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
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

            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

#[instrument(name = "sim_bootstrap.interest_late.process_obligation_message", skip(message, cf_proposal, tx), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
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

            tx.send((*obligation_type, *amount)).await?;
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
    name = "sim_bootstrap.do_interest_late",
    skip(app, obligation_amount_rx),
    err
)]
async fn do_interest_late(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<(ObligationType, UsdCents)>,
) -> anyhow::Result<()> {
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);
    let mut month_num = 0;
    let mut first_interest = UsdCents::ZERO;

    while let Some((obligation_type, amount)) = obligation_amount_rx.recv().await {
        if month_num < 3 {
            month_num += 1;
            sim_time::sleep(one_month).await;
        }

        if obligation_type == ObligationType::Interest && first_interest.is_zero() {
            first_interest = amount;
            continue;
        }

        app.record_payment_with_date(&sub, id, amount, sim_time::now().date_naive())
            .await?;

        let facility = app
            .credit()
            .facilities()
            .find_by_id(&sub, id)
            .await?
            .unwrap();
        let total_outstanding = app.credit().outstanding(&facility).await?;
        if total_outstanding == first_interest {
            break;
        }
    }

    // Delay first payment by 1 more month
    sim_time::sleep(one_month).await;

    app.record_payment_with_date(&sub, id, first_interest, sim_time::now().date_naive())
        .await?;

    if app
        .credit()
        .facilities()
        .has_outstanding_obligations(&sub, id)
        .await?
    {
        while let Some((_, amount)) = obligation_amount_rx.recv().await {
            app.record_payment_with_date(&sub, id, amount, sim_time::now().date_naive())
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
    }

    app.credit().complete_facility(&sub, id).await?;

    Ok(())
}
