mod helpers;

use authz::dummy::DummySubject;

use core_credit::*;
use core_credit_collection::{CollectionLedgerError, PaymentError};

use core_credit::error::CoreCreditError;
use money::UsdCents;

/// Test that attempting to pay more than the outstanding obligations returns
/// the `PaymentAmountGreaterThanOutstandingObligations` error.
#[tokio::test]
#[serial_test::file_serial(job_poller)]
async fn payment_exceeding_obligations_returns_error() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let state = helpers::create_active_facility(&ctx, helpers::test_terms()).await?;

    let facility_id = state.facility_id;
    let deposit_account_id = state.deposit_account_id;

    // Attempt overpayment and verify error
    let payment_amount = UsdCents::from(100);
    ctx.deposit
        .record_deposit(&DummySubject, deposit_account_id, payment_amount, None)
        .await?;
    let result = ctx
        .credit
        .record_payment(
            &DummySubject,
            facility_id,
            PaymentSourceAccountId::new(deposit_account_id.into()),
            payment_amount,
        )
        .await;
    assert!(result.is_ok());

    let facility_amount = state.amount;
    ctx.deposit
        .record_deposit(&DummySubject, deposit_account_id, facility_amount, None)
        .await?;
    let result = ctx
        .credit
        .record_payment(
            &DummySubject,
            facility_id,
            PaymentSourceAccountId::new(deposit_account_id.into()),
            facility_amount,
        )
        .await;
    assert!(
        matches!(
            result,
            Err(CoreCreditError::PaymentError(
                PaymentError::CollectionLedgerError(
                    CollectionLedgerError::PaymentAmountGreaterThanOutstandingObligations,
                )
            )),
        ),
        "{}",
        match &result {
            Err(e) => format!("{}", e),
            Ok(f) => format!("Credit Facility: {}", f.id),
        },
    );

    ctx.jobs.shutdown().await?;
    Ok(())
}
