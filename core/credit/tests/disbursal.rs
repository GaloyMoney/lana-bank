mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;

/// `DisbursalSettled` is published when the initial disbursal settles during
/// facility activation.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
/// (which drives activation and disbursal approval/settlement jobs)
///
/// # Consumers
/// - `History::process_credit_event` - records disbursal execution
/// - `RepaymentPlan::process_credit_event` - updates disbursal/obligation plan state
/// - Admin/API subscribers tracking disbursal settlement
///
/// # Event Contents
/// - `credit_facility_id`: Activated facility identifier
/// - `amount`: Disbursed amount
/// - `settlement`: Settlement metadata (`tx_id`, `effective`)
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn disbursal_settled_event_on_activation() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_pending_facility(&ctx, helpers::test_terms()).await?;

    let collateral_satoshis = money::Satoshis::from(50_000_000); // 0.5 BTC
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
            CoreCreditEvent::DisbursalSettled { entity }
                if entity.credit_facility_id == CreditFacilityId::from(pending_facility_id) =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(
        recorded.credit_facility_id,
        CreditFacilityId::from(state.pending_facility_id)
    );
    assert_eq!(recorded.amount, state.amount);
    let settlement = recorded.settlement.expect("settlement should be present");
    assert_eq!(settlement.effective, effective);

    ctx.jobs.shutdown().await?;
    Ok(())
}
