mod helpers;

use audit::SystemSubject;
use authz::dummy::DummySubject;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollectionEvent, PaymentId, PaymentLedgerAccountIds,
    PaymentSourceAccountId,
};
use helpers::event::{DummyEvent, expect_event};
use money::UsdCents;

/// `PaymentCreated` is published when a payment is recorded.
///
/// # Trigger
/// `Payments::record`
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - allocates payment to obligations
/// - `CreditFacilityRepaymentPlanJob` - triggers repayment plan rebuild
/// - Dagster dbt pipeline
///
/// # Event Contents
/// - `id`: Unique payment identifier
/// - `beneficiary_id`: Beneficiary identifier
/// - `amount`: Payment amount
/// - `recorded_at`: Timestamp of payment recording
/// - `effective`: Effective date of payment
#[tokio::test]
async fn payment_created_event_on_record() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);

    let payment_ledger_accounts = PaymentLedgerAccountIds {
        facility_payment_holding_account_id: ctx.accounts.payment_holding,
        facility_uncovered_outstanding_account_id: ctx.accounts.uncovered_outstanding,
        payment_source_account_id: PaymentSourceAccountId::new(ctx.accounts.payment_source),
    };

    let payment_id = PaymentId::new();
    let payments = ctx.collections.payments().clone();
    let effective = ctx.clock.today();

    let (payment, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let payments = payments.clone();
            async move {
                payments
                    .record(
                        payment_id,
                        beneficiary_id,
                        payment_ledger_accounts,
                        amount,
                        effective,
                        &DummySubject::system(),
                    )
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("payment was not created"))
            }
        },
        |result, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::PaymentCreated {
                entity,
            }) if entity.id == result.id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, payment.id);
    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.amount, amount);

    Ok(())
}
