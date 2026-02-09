mod helpers;

use audit::SystemSubject;
use authz::dummy::DummySubject;
use cala_ledger::primitives::TransactionId as LedgerTxId;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollectionEvent, NewObligation, ObligationId, ObligationType,
    PaymentDetailsForAllocation, PaymentId, PaymentLedgerAccountIds, PaymentSourceAccountId,
};
use core_credit_terms::EffectiveDate;
use es_entity::DbOp;
use helpers::event::{DummyEvent, expect_event};
use money::UsdCents;

use helpers::TestContext;

async fn create_obligation(ctx: &TestContext, beneficiary_id: BeneficiaryId, amount: UsdCents) {
    let due_date: EffectiveDate = ctx
        .clock
        .today()
        .checked_add_days(chrono::Days::new(30))
        .expect("due date overflow")
        .into();
    let overdue_date = due_date.checked_add_days(chrono::Days::new(30));
    let new_obligation = NewObligation::builder()
        .id(ObligationId::new())
        .tx_id(LedgerTxId::new())
        .beneficiary_id(beneficiary_id)
        .obligation_type(ObligationType::Disbursal)
        .amount(amount)
        .receivable_account_ids(ctx.accounts.receivable)
        .defaulted_account_id(ctx.accounts.defaulted)
        .due_date(due_date)
        .overdue_date(overdue_date)
        .liquidation_date(None)
        .effective(ctx.clock.today())
        .build()
        .expect("could not build new obligation");

    let mut op = DbOp::init_with_clock(&ctx.pool, &ctx.clock).await.unwrap();
    ctx.collections
        .obligations()
        .create_with_jobs_in_op(&mut op, new_obligation)
        .await
        .expect("could not create obligation");
    op.commit().await.unwrap();
}

/// `PaymentAllocationCreated` is published when a payment is allocated to an obligation.
///
/// # Trigger
/// `Obligations::allocate_payment_in_op` creates an allocation for each obligation.
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - updates repayment entry
/// - `CreditFacilityRepaymentPlanJob` - triggers repayment plan rebuild
/// - Dagster dbt pipeline
///
/// # Event Contents
/// - `id`: Unique allocation identifier
/// - `obligation_id`: The obligation the payment was allocated to
/// - `obligation_type`: `Disbursal` or `Interest`
/// - `beneficiary_id`: Beneficiary identifier
/// - `amount`: Allocated amount
/// - `recorded_at`: Timestamp of allocation creation
/// - `effective`: Effective date of allocation
#[tokio::test]
async fn payment_allocation_created_event_on_allocate() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);

    // Create an obligation first
    create_obligation(&ctx, beneficiary_id, amount).await;

    // Record a payment
    let payment_ledger_accounts = PaymentLedgerAccountIds {
        facility_payment_holding_account_id: ctx.accounts.payment_holding,
        facility_uncovered_outstanding_account_id: ctx.accounts.uncovered_outstanding,
        payment_source_account_id: PaymentSourceAccountId::new(ctx.accounts.payment_source),
    };

    let payment = ctx
        .collections
        .payments()
        .record(
            PaymentId::new(),
            beneficiary_id,
            payment_ledger_accounts,
            amount,
            ctx.clock.today(),
            &DummySubject::system(),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("payment was not created"))?;

    let payment_details = PaymentDetailsForAllocation::from(payment);
    let obligations = ctx.collections.obligations().clone();
    let pool = ctx.pool.clone();
    let clock = ctx.clock.clone();

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            let mut op = DbOp::init_with_clock(&pool, &clock).await?;
            obligations
                .allocate_payment_in_op(&mut op, payment_details, &DummySubject::system())
                .await?;
            op.commit().await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCreditCollection(
                CoreCreditCollectionEvent::PaymentAllocationCreated { entity },
            ) if entity.beneficiary_id == beneficiary_id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.obligation_type, ObligationType::Disbursal);
    assert_eq!(recorded.amount, amount);

    Ok(())
}
