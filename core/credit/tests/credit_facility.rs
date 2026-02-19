mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;
use money::{Satoshis, UsdCents};
use std::time::Duration;

/// `FacilityActivated` is published when a pending facility transitions to active.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
/// (which drives collateralization checks and activation jobs)
///
/// # Consumers
/// - `History::process_credit_event` - records facility approval/activation
/// - `RepaymentPlan::process_credit_event` - sets activation time for plan projection
/// - Admin/API subscribers waiting for activated facilities
///
/// # Event Contents
/// - `id`: Credit facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Approved facility amount
/// - `completed_at`: Completion timestamp (none when activated)
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_activated_event_on_activation() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_pending_facility(&ctx, helpers::test_terms()).await?;

    let collateral_satoshis = Satoshis::from(50_000_000);
    let effective = chrono::Utc::now().date_naive();

    let collaterals = ctx.credit.collaterals().clone();
    let collateral_id = state.collateral_id;
    let pending_facility_id = state.pending_facility_id;
    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let collaterals = collaterals.clone();
            async move {
                collaterals
                    .update_collateral_by_id(
                        &DummySubject,
                        collateral_id,
                        collateral_satoshis,
                        effective,
                    )
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityActivated { entity }
                if CreditFacilityId::from(pending_facility_id) == entity.id =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(
        recorded.id,
        CreditFacilityId::from(state.pending_facility_id)
    );
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    assert!(recorded.completed_at.is_none());

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// `FacilityCompleted` is published when an active facility is explicitly completed
/// after obligations are cleared.
///
/// # Trigger
/// `CoreCredit::complete_facility`
///
/// # Consumers
/// - `History::process_credit_event` - records completion
/// - `RepaymentPlan::process_credit_event` - finalizes plan state
/// - Admin/API subscribers for completed facilities
///
/// # Event Contents
/// - `id`: Credit facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Facility amount
/// - `completed_at`: Completion timestamp
/// - `liquidation_trigger`: Optional liquidation context (none for normal completion)
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_completed_event_on_completion() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_active_facility(&ctx, helpers::test_terms()).await?;

    // Pay off all outstanding obligations.
    let payment_amount = state.amount;
    ctx.deposit
        .record_deposit(
            &DummySubject,
            state.deposit_account_id,
            payment_amount,
            None,
        )
        .await?;
    ctx.credit
        .record_payment(
            &DummySubject,
            state.facility_id,
            PaymentSourceAccountId::new(state.deposit_account_id.into()),
            payment_amount,
        )
        .await?;

    // Wait for payment allocation job to clear all obligations.
    for attempt in 0..100 {
        let balances = ctx
            .credit
            .facilities()
            .balance(&DummySubject, state.facility_id)
            .await?;
        if !balances.any_outstanding_or_defaulted() {
            break;
        }
        if attempt == 99 {
            panic!("Timed out waiting for payment allocation after 5 seconds");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let credit = ctx.credit.clone();
    let facility_id = state.facility_id;
    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let credit = credit.clone();
            async move { credit.complete_facility(&DummySubject, facility_id).await }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityCompleted { entity } if entity.id == facility_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, state.facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    assert!(recorded.completed_at.is_some());
    assert!(recorded.liquidation_trigger.is_none());

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// `PartialLiquidationInitiated` is published when facility collateralization
/// drops below liquidation thresholds and a partial liquidation starts.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
/// with collateral reduced below liquidation CVL
///
/// # Consumers
/// - `History::process_credit_event` - records liquidation lifecycle
/// - `RepaymentPlan::process_credit_event` - updates obligations affected by liquidation
/// - Collateral/liquidation jobs that move collateral and receive proceeds
///
/// # Event Contents
/// - `id`: Credit facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Facility amount
/// - `liquidation_trigger`: Estimated liquidation and proceeds requirements
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn partial_liquidation_initiated_event_on_undercollateralization() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_active_facility(&ctx, helpers::test_terms()).await?;

    // Wait for the ledger to reflect the disbursal.
    for attempt in 0..100 {
        let balances = ctx
            .credit
            .facilities()
            .balance(&DummySubject, state.facility_id)
            .await?;
        if balances.any_outstanding_or_defaulted() {
            break;
        }
        if attempt == 99 {
            panic!("Timed out waiting for outstanding balance to appear");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Reduce collateral far below the liquidation CVL threshold (105%).
    let tiny_collateral = Satoshis::from(1_000_000);
    let effective = chrono::Utc::now().date_naive();

    let collaterals = ctx.credit.collaterals().clone();
    let collateral_id = state.collateral_id;
    let facility_id = state.facility_id;
    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let collaterals = collaterals.clone();
            async move {
                collaterals
                    .update_collateral_by_id(
                        &DummySubject,
                        collateral_id,
                        tiny_collateral,
                        effective,
                    )
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::PartialLiquidationInitiated { entity } if entity.id == facility_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, state.facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    let trigger = recorded
        .liquidation_trigger
        .expect("liquidation_trigger should be present");
    assert!(trigger.initially_expected_to_receive > UsdCents::ZERO);
    assert!(trigger.initially_estimated_to_liquidate > Satoshis::ZERO);

    ctx.jobs.shutdown().await?;
    Ok(())
}
