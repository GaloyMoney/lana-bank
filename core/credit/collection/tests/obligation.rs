//! TODO: Missing tests for `ObligationDue`, `ObligationOverdue`, and `ObligationDefaulted`.

mod helpers;

use cala_ledger::primitives::TransactionId as LedgerTxId;
use core_accounting::LedgerTransactionInitiator;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollectionEvent, NewObligation, Obligation, ObligationId,
    ObligationType, PaymentDetailsForAllocation, PaymentId, PaymentLedgerAccountIds,
    PaymentSourceAccountId,
};
use core_credit_terms::EffectiveDate;
use core_money::UsdCents;
use es_entity::DbOp;
use helpers::event::{DummyEvent, expect_event};

use helpers::TestContext;

async fn create_obligation(
    ctx: &TestContext,
    beneficiary_id: BeneficiaryId,
    amount: UsdCents,
) -> anyhow::Result<Obligation> {
    let due_date: EffectiveDate = ctx.clock.today().into();
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

    let mut op = DbOp::init_with_clock(&ctx.pool, &ctx.clock).await?;
    let obligation = ctx
        .collections
        .obligations()
        .create_with_jobs_in_op(&mut op, new_obligation)
        .await?;
    op.commit().await?;

    Ok(obligation)
}

/// `ObligationCreated` is published when a new obligation is created.
///
/// # Trigger
/// `Obligations::create_with_jobs_in_op`
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - adds a new repayment entry
/// - `CreditFacilityRepaymentPlanJob` - triggers repayment plan rebuild
/// - `credit_facility::collateralization_from_events` - updates collateralization state
/// - Dagster dbt pipeline - `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `id`: Unique obligation identifier
/// - `obligation_type`: `Disbursal` or `Interest`
/// - `beneficiary_id`: Beneficiary identifier
/// - `initial_amount`: Original obligation amount
/// - `outstanding_amount`: Current amount owed
/// - `due_at`, `overdue_at`, `defaulted_at`: Lifecycle dates
#[tokio::test]
async fn obligation_created_event_on_create() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);

    let (_, recorded) = expect_event(
        &ctx.outbox,
        || create_obligation(&ctx, beneficiary_id, amount),
        |result, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationCreated {
                entity,
            }) if entity.id == result.id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.obligation_type, ObligationType::Disbursal);
    assert_eq!(recorded.initial_amount, amount);
    assert_eq!(recorded.outstanding_amount, amount);

    Ok(())
}

/// `ObligationCompleted` is published when an obligation is fully paid off.
///
/// # Trigger
/// `Obligations::allocate_payment_in_op` reduces outstanding to zero.
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - marks repayment entry as `Paid`
/// - `CreditFacilityRepaymentPlanJob` - triggers repayment plan rebuild
/// - Dagster dbt pipeline - `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `outstanding_amount`: Zero
/// - All other fields from `ObligationCreated`
#[tokio::test]
async fn obligation_completed_event_on_full_payment() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let obligation = create_obligation(&ctx, beneficiary_id, amount).await?;

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
            LedgerTransactionInitiator::System,
        )
        .await?
        .expect("payment should be created");

    let payment_details = PaymentDetailsForAllocation::from(payment);
    let obligations = ctx.collections.obligations().clone();
    let pool = ctx.pool.clone();
    let clock = ctx.clock.clone();
    let obligation_id = obligation.id;

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            let mut op = DbOp::init_with_clock(&pool, &clock).await?;
            obligations
                .allocate_payment_in_op(
                    &mut op,
                    payment_details,
                    LedgerTransactionInitiator::System,
                )
                .await?;
            op.commit().await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationCompleted {
                entity,
            }) if entity.id == obligation_id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.outstanding_amount, UsdCents::ZERO);
    assert_eq!(recorded.initial_amount, amount);

    Ok(())
}
