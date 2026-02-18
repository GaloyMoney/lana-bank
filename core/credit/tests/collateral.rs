mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;
use money::Satoshis;

/// `FacilityCollateralUpdated` is published when collateral is manually updated.
///
/// # Trigger
/// `Collaterals::update_collateral_by_id`
///
/// # Consumers
/// - `History::process_credit_event` - records collateral adjustments
/// - `CreditFacilityCollateralizationFromEventsJob` - recalculates collateralization state
/// - Dashboard/API subscribers that track collateral movement
///
/// # Event Contents
/// - `id`: Collateral entity identifier
/// - `pending_credit_facility_id`: Pending facility linked to the collateral
/// - `amount`: Updated collateral amount
/// - `adjustment`: Ledger adjustment details for the update
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_collateral_updated_event_on_manual_update() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_pending_facility(&ctx, helpers::test_terms()).await?;

    let collateral_satoshis = Satoshis::from(1_000_000);
    let effective = chrono::Utc::now().date_naive();

    let (collateral, recorded) = expect_event(
        &ctx.outbox,
        || {
            ctx.credit.collaterals().update_collateral_by_id(
                &DummySubject,
                state.collateral_id,
                collateral_satoshis,
                effective,
            )
        },
        |result, e| match e {
            CoreCreditEvent::FacilityCollateralUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, collateral.id);
    assert_eq!(state.pending_facility_id, recorded.secured_loan_id.into());
    assert_eq!(recorded.amount, collateral_satoshis);
    ctx.jobs.shutdown().await?;

    Ok(())
}
