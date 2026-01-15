use futures::StreamExt;
use lana_app::{app::LanaApp, primitives::*};
use lana_events::{CoreCreditEvent, LanaEvent};
use obix::out::PersistentOutboxEvent;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use tracing::{Instrument, Span, instrument};

use crate::helpers;

// Scenario 4: A credit facility that has multiple disbursals making timely payments
#[tracing::instrument(
    name = "sim_bootstrap.disbursal_different_months_scenario",
    skip(app),
    err
)]
pub async fn disbursal_different_months_scenario(
    sub: Subject,
    app: &LanaApp,
) -> anyhow::Result<()> {
    let (customer_id, _) =
        helpers::create_customer(&sub, app, "4-disbursal-different-months").await?;

    let deposit_amount = UsdCents::try_from_usd(dec!(10_000_000))?;
    helpers::make_deposit(&sub, app, &customer_id, deposit_amount).await?;

    let cf_terms = helpers::std_terms_12m();
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

    let sim_app = app.clone();
    tokio::spawn(
        async move {
            do_disbursal_in_different_months(sub, sim_app, cf_proposal.id.into())
                .await
                .expect("disbursal different months failed");
        }
        .instrument(Span::current()),
    );

    let (tx, rx) = mpsc::channel::<UsdCents>(32);
    let sim_app = app.clone();
    tokio::spawn(
        async move {
            do_timely_payments(sub, sim_app, cf_proposal.id.into(), rx)
                .await
                .expect("disbursal different months timely payments failed");
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

#[instrument(name = "sim_bootstrap.disbursal_different_months.process_activation_message", skip(message, sub, app, cf_proposal), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_activation_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    sub: &Subject,
    app: &LanaApp,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
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
                    cf_proposal.id,
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

#[instrument(name = "sim_bootstrap.disbursal_different_months.process_obligation_message", skip(message, cf_proposal, tx), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
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
    name = "sim_bootstrap.do_disbursal_in_different_months",
    skip(app),
    err
)]
async fn do_disbursal_in_different_months(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
) -> anyhow::Result<()> {
    let one_month = std::time::Duration::from_secs(30 * 24 * 60 * 60);

    // there is already one disbursal in month 1
    sim_time::sleep(one_month).await;

    // disbursal in month 2
    app.credit()
        .initiate_disbursal(&sub, id, UsdCents::try_from_usd(dec!(2_000_000))?)
        .await?;

    sim_time::sleep(one_month * 2).await;

    // disbursal in month 3
    app.credit()
        .initiate_disbursal(&sub, id, UsdCents::try_from_usd(dec!(5_000_000))?)
        .await?;

    Ok(())
}

#[tracing::instrument(
    name = "sim_bootstrap.disbursal_different_months.do_timely_payments",
    skip(app, obligation_amount_rx),
    err
)]
async fn do_timely_payments(
    sub: Subject,
    app: LanaApp,
    id: CreditFacilityId,
    mut obligation_amount_rx: mpsc::Receiver<UsdCents>,
) -> anyhow::Result<()> {
    while let Some(amount) = obligation_amount_rx.recv().await {
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

    app.credit().complete_facility(&sub, id).await?;

    Ok(())
}
