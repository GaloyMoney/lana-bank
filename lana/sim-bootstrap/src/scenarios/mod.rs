mod disbursal_different_months;
mod interest_late;
mod interest_under_payment;
mod principal_late;
mod principal_under_payment;
mod timely_payments;

use es_entity::clock::{ClockController, ClockHandle};
use futures::StreamExt;
use rust_decimal_macros::dec;
use tracing::{Span, instrument};

use lana_app::{app::LanaApp, primitives::*};
use lana_events::*;
use obix::out::PersistentOutboxEvent;

use super::helpers;

#[instrument(
    name = "sim_bootstrap.scenarios.run",
    skip(app, clock, clock_ctrl),
    err
)]
pub async fn run(
    sub: &Subject,
    app: &LanaApp,
    clock: ClockHandle,
    clock_ctrl: ClockController,
) -> anyhow::Result<(), anyhow::Error> {
    timely_payments::timely_payments_scenario(sub.clone(), app, &clock, &clock_ctrl).await?;
    interest_late::interest_late_scenario(sub.clone(), app, &clock, &clock_ctrl).await?;
    principal_late::principal_late_scenario(sub.clone(), app, &clock, &clock_ctrl).await?;
    disbursal_different_months::disbursal_different_months_scenario(
        sub.clone(),
        app,
        &clock,
        &clock_ctrl,
    )
    .await?;
    principal_under_payment::principal_under_payment_scenario(
        sub.clone(),
        app,
        &clock,
        &clock_ctrl,
    )
    .await?;
    interest_under_payment::interest_under_payment_scenario(sub.clone(), app, &clock, &clock_ctrl)
        .await?;

    Ok(())
}

#[instrument(name = "sim_bootstrap.process_facility_lifecycle", skip(sub, app, clock), fields(customer_id = %customer_id, deposit_account_id = %deposit_account_id, proposal_id = tracing::field::Empty))]
pub async fn process_facility_lifecycle(
    sub: Subject,
    app: LanaApp,
    customer_id: CustomerId,
    deposit_account_id: DepositAccountId,
    clock: ClockHandle,
) -> anyhow::Result<()> {
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
        .await?;

    while let Some(msg) = stream.next().await {
        if process_facility_message(&msg, &sub, &app, &cf_proposal, &clock).await? {
            break;
        }
    }

    Ok(())
}

#[instrument(name = "sim_bootstrap.process_facility_message", skip(message, sub, app, cf_proposal, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
async fn process_facility_message(
    message: &PersistentOutboxEvent<LanaEvent>,
    sub: &Subject,
    app: &LanaApp,
    cf_proposal: &lana_app::credit::CreditFacilityProposal,
    clock: &ClockHandle,
) -> anyhow::Result<bool> {
    match &message.payload {
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityProposalConcluded { entity }))
            if entity.status == CreditFacilityProposalStatus::Approved
                && cf_proposal.id == entity.id =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            let pending_facility = app
                .credit()
                .pending_credit_facilities()
                .find_by_id(sub, cf_proposal.id)
                .await?
                .expect("pending facility exists");

            app.credit()
                .collaterals()
                .update_collateral_by_id(
                    sub,
                    pending_facility.collateral_id,
                    Satoshis::try_from_btc(dec!(230))?,
                    clock.today(),
                )
                .await?;
        }
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityActivated { entity }))
            if entity.id == cf_proposal.id.into() =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            app.credit()
                .initiate_disbursal(sub, entity.id, UsdCents::try_from_usd(dec!(1_000_000))?)
                .await?;
        }
        Some(LanaEvent::CreditCollection(
            event @ CoreCreditCollectionEvent::ObligationDue { entity },
        )) if {
            let id: CreditFacilityId = entity.beneficiary_id.into();
            id == cf_proposal.id.into() && entity.outstanding_amount > UsdCents::ZERO
        } =>
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            let id: CreditFacilityId = entity.beneficiary_id.into();
            let _ = app
                .record_payment_with_date(sub, id, entity.outstanding_amount, clock.today())
                .await;
            let facility = app
                .credit()
                .facilities()
                .find_by_id(sub, id)
                .await?
                .expect("cf exists");
            if facility.interest_accrual_cycle_in_progress().is_none() {
                let total_outstanding_amount = app.credit().outstanding(&facility).await?;
                app.record_payment_with_date(
                    sub,
                    facility.id,
                    total_outstanding_amount,
                    clock.today(),
                )
                .await?;
                app.credit().complete_facility(sub, facility.id).await?;
            }
        }
        Some(LanaEvent::Credit(event @ CoreCreditEvent::FacilityCompleted { entity })) => {
            if entity.id == cf_proposal.id.into() {
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
