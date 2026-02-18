mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;
use money::Satoshis;

/// `PendingCreditFacilityCompleted` is published when a pending facility is
/// completed as part of activation.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
/// (which drives pending-facility collateralization and completion jobs)
///
/// # Consumers
/// - `History::process_credit_event` - records pending facility lifecycle updates
/// - `RepaymentPlan::process_credit_event` - updates repayment plan initialization state
/// - Admin/API subscribers waiting for pending facility completion
///
/// # Event Contents
/// - `id`: Pending facility identifier
/// - `customer_id`: Facility owner
/// - `amount`: Facility amount
/// - `terms`: Facility terms snapshot
/// - `status`: Pending facility status (`Completed`)
/// - `completed_at`: Completion timestamp
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn pending_credit_facility_completed_event_on_activation() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_pending_facility(&ctx, helpers::test_terms()).await?;

    let collateral_satoshis = Satoshis::from(50_000_000); // 0.5 BTC
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
            CoreCreditEvent::PendingCreditFacilityCompleted { entity }
                if entity.id == pending_facility_id =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, state.pending_facility_id);
    assert_eq!(recorded.customer_id, state.customer_id);
    assert_eq!(recorded.amount, state.amount);
    assert_eq!(recorded.terms, state.terms);
    assert_eq!(recorded.status, PendingCreditFacilityStatus::Completed);
    assert!(recorded.completed_at.is_some());

    ctx.jobs.shutdown().await?;
    Ok(())
}
