mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;
use money::{Satoshis, UsdCents};
use rust_decimal_macros::dec;
use std::time::Duration;

fn isolated_price_change_terms() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(300))
        .margin_call_cvl(dec!(280))
        .liquidation_cvl(dec!(200))
        .duration(FacilityDuration::Months(3))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
        .obligation_liquidation_duration_from_due(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::SingleDisbursal)
        .build()
        .unwrap()
}

/// `FacilityCollateralizationChanged` is published when an active facility's
/// collateralization state changes due to a collateral update.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
/// → `CollateralUpdated` event
/// → `CreditFacilityCollateralizationFromEventsHandler::update_collateralization_from_events`
///
/// # Consumers
/// - `History::process_credit_event` - records collateralization state changes
/// - `RepaymentPlan::process_credit_event` - updates plan projections
/// - Admin GraphQL subscription - pushes real-time updates
///
/// # Event Contents
/// - `id`: Credit facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Facility amount
/// - `collateralization`: Updated collateralization state, collateral, outstanding, and price
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_collateralization_changed_event_on_collateral_update() -> anyhow::Result<()> {
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

    // Reduce collateral below margin call CVL (125%) but above liquidation CVL (105%).
    // Facility amount is $10,000 at BTC price $70,000.
    // 16,000,000 sats = 0.16 BTC ≈ $11,200 → ~112% CVL = UnderMarginCallThreshold.
    let reduced_collateral = Satoshis::from(16_000_000);
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
                        reduced_collateral,
                        effective,
                    )
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityCollateralizationChanged { entity }
                if entity.id == facility_id =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, state.facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    assert_eq!(
        recorded.collateralization.state,
        CollateralizationState::UnderMarginCallThreshold,
    );
    assert_eq!(recorded.collateralization.collateral, reduced_collateral);

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// `FacilityCollateralizationChanged` is published when an active facility's
/// collateralization state changes due to a BTC price drop.
///
/// # Trigger
/// `CorePriceEvent::PriceUpdated` (ephemeral)
/// → `CreditFacilityCollateralizationFromEventsHandler::update_collateralization_from_price_event`
///
/// # Consumers
/// - `History::process_credit_event` - records collateralization state changes
/// - `RepaymentPlan::process_credit_event` - updates plan projections
/// - Admin GraphQL subscription - pushes real-time updates
///
/// # Event Contents
/// - `id`: Credit facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Facility amount
/// - `collateralization`: Updated collateralization state with new price
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_collateralization_changed_event_on_price_change() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_active_facility(&ctx, isolated_price_change_terms()).await?;

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

    // Use high CVL thresholds to avoid changing collateralization for most facilities
    // created by other tests:
    // - This test facility uses margin_call=280%, liquidation=200%.
    // - At $55,000/BTC: 0.5 BTC ≈ $27,500; with ~$10,000 outstanding => ~275% CVL.
    //   This is below 280% (so it transitions to UnderMarginCallThreshold) but above 200%.
    // - Facilities with default terms (margin_call=125%) remain fully collateralized.
    let low_price = core_price::PriceOfOneBTC::new(money::UsdCents::from(5_500_000));

    let outbox = ctx.outbox.clone();
    let facility_id = state.facility_id;
    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let outbox = outbox.clone();
            async move {
                outbox
                    .publish_ephemeral(
                        core_price::PRICE_UPDATED_EVENT_TYPE,
                        core_price::CorePriceEvent::PriceUpdated {
                            price: low_price,
                            timestamp: chrono::Utc::now(),
                        },
                    )
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityCollateralizationChanged { entity }
                if entity.id == facility_id =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, state.facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    assert_eq!(
        recorded.collateralization.state,
        CollateralizationState::UnderMarginCallThreshold,
    );
    assert_eq!(recorded.collateralization.price_at_state_change, low_price);

    ctx.jobs.shutdown().await?;
    Ok(())
}

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
#[serial_test::file_serial(job_poller)]
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
#[serial_test::file_serial(job_poller)]
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
#[serial_test::file_serial(job_poller)]
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
