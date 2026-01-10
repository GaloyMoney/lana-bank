mod disbursal_different_months;
mod interest_late;
mod interest_under_payment;
mod principal_late;
mod principal_under_payment;
mod timely_payments;

use futures::StreamExt;
use rust_decimal_macros::dec;
use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use lana_app::{app::LanaApp, credit::error::CoreCreditError, primitives::*};
use lana_events::*;
use obix::out::PersistentOutboxEvent;
use tokio::task::JoinHandle;

use super::error::SimBootstrapError;
use super::helpers;

#[record_error_severity]
#[instrument(name = "sim_bootstrap.scenarios.run", skip(app), err)]
pub async fn run(
    sub: &Subject,
    app: &LanaApp,
) -> Result<Vec<JoinHandle<Result<(), SimBootstrapError>>>, SimBootstrapError> {
    let mut handles = Vec::new();
    let sub = *sub;

    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            timely_payments::timely_payments_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            interest_late::interest_late_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            principal_late::principal_late_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            disbursal_different_months::disbursal_different_months_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            interest_under_payment::interest_under_payment_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            principal_under_payment::principal_under_payment_scenario(sub, &app).await
        }));
    }

    Ok(handles)
}

#[instrument(name = "sim_bootstrap.process_facility_lifecycle", skip(sub, app), fields(customer_id = %customer_id, deposit_account_id = %deposit_account_id, proposal_id = tracing::field::Empty))]
pub async fn process_facility_lifecycle(
    sub: Subject,
    app: LanaApp,
    customer_id: CustomerId,
    deposit_account_id: DepositAccountId,
) -> Result<(), SimBootstrapError> {
    let terms = helpers::std_terms();

    let mut stream = app.outbox().listen_persisted(None);

    let cf_proposal = app
        .create_facility_proposal(
            &sub,
            customer_id,
            UsdCents::try_from_usd(dec!(10_000_000))?,
            terms,
            None::<CustodianId>,
        )
        .await?;

    Span::current().record("proposal_id", tracing::field::display(cf_proposal.id));

    let cf_proposal = app
        .credit()
        .proposals()
        .conclude_customer_approval(&sub, cf_proposal.id, true)
        .await
        .map_err(CoreCreditError::from)?;

    while let Some(msg) = stream.next().await {
        if process_facility_message(&msg, &sub, &app, &cf_proposal).await? {
            break;
        }
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.process_facility_message", skip(message, sub, app, cf_proposal), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_facility_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    sub: &Subject,
    app: &LanaApp,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
) -> Result<bool, SimBootstrapError> {
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
        }
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

            let _ = app
                .record_payment_with_date(sub, *id, *amount, sim_time::now().date_naive())
                .await;
            let facility = app
                .credit()
                .facilities()
                .find_by_id(sub, *id)
                .await
                .map_err(CoreCreditError::from)?
                .expect("cf exists");
            if facility.interest_accrual_cycle_in_progress().is_none() {
                let total_outstanding_amount = app.credit().outstanding(&facility).await?;
                app.record_payment_with_date(
                    sub,
                    facility.id,
                    total_outstanding_amount,
                    sim_time::now().date_naive(),
                )
                .await?;
                app.credit().complete_facility(sub, facility.id).await?;
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
