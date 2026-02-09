mod helpers;

use audit::SystemSubject;
use authz::dummy::DummySubject;
use cala_ledger::primitives::TransactionId as LedgerTxId;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollectionEvent, NewObligation, Obligation, ObligationId,
    ObligationType, PaymentDetailsForAllocation, PaymentId, PaymentLedgerAccountIds,
    PaymentSourceAccountId,
};
use core_credit_terms::EffectiveDate;
use es_entity::DbOp;
use helpers::event::{DummyEvent, expect_event};
use money::UsdCents;

use helpers::TestContext;

async fn create_obligation_with_dates(
    ctx: &TestContext,
    beneficiary_id: BeneficiaryId,
    amount: UsdCents,
    due_date: EffectiveDate,
    overdue_date: Option<EffectiveDate>,
    defaulted_date: Option<EffectiveDate>,
) -> anyhow::Result<Obligation> {
    // Build with explicit lifecycle dates so tests can drive status transitions deterministically.
    let mut builder = NewObligation::builder();
    builder
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
        .effective(ctx.clock.today());

    // Only set defaulted_date when the test expects the defaulted transition.
    if let Some(defaulted_date) = defaulted_date {
        builder.defaulted_date(defaulted_date);
    }

    let new_obligation = builder.build().expect("could not build new obligation");

    let mut op = DbOp::init_with_clock(&ctx.pool, &ctx.clock).await?;
    let obligation = ctx
        .collections
        .obligations()
        .create_with_jobs_in_op(&mut op, new_obligation)
        .await?;
    op.commit().await?;

    Ok(obligation)
}

async fn create_obligation(
    ctx: &TestContext,
    beneficiary_id: BeneficiaryId,
    amount: UsdCents,
) -> anyhow::Result<Obligation> {
    let due_date: EffectiveDate = ctx
        .clock
        .today()
        .checked_add_days(chrono::Days::new(30))
        .expect("due date overflow")
        .into();
    let overdue_date = due_date.checked_add_days(chrono::Days::new(30));
    create_obligation_with_dates(ctx, beneficiary_id, amount, due_date, overdue_date, None).await
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

/// `ObligationDue` is published when an obligation moves to the due state.
///
/// # Trigger
/// `ObligationDueJobRunner::record_due` (scheduled by `Obligations::create_withjobs_in_op`)
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - marks repayment entry as `Due`
/// - `CreditFacilityRepaymentPlanJob` - rebuilds the repayment plan projection
/// - Dagster dbt pipeline - `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `id`: Unique obligation identifier
/// - `beneficiary_id`: Beneficiary identifier
/// - `initial_amount`: Original obligation amount
/// - `outstanding_amount`: Current amount owed
/// - `due_at`: Effective due date
#[tokio::test]
async fn obligation_due_event_on_due_job() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    // The due/overdue/defaulted jobs run via the poller.
    ctx.jobs.start_poll().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    // Due today so the scheduled job fires immediately.
    let due_date: EffectiveDate = ctx.clock.today().into();

    let (obligation, recorded) = expect_event(
        &ctx.outbox,
        || create_obligation_with_dates(&ctx, beneficiary_id, amount, due_date, None, None),
        |result, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationDue {
                entity,
            }) if entity.id == result.id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.initial_amount, amount);
    assert_eq!(recorded.outstanding_amount, amount);
    assert_eq!(recorded.due_at, due_date);
    assert!(recorded.overdue_at.is_none());
    assert!(recorded.defaulted_at.is_none());
    assert_eq!(recorded.id, obligation.id);

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// `ObligationOverdue` is published when an obligation moves to the overdue state.
///
/// # Trigger
/// `ObligationOverdueJobRunner::record_overdue` (scheduled by `ObligationDueJobRunner::record_due`)
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - marks repayment entry as `Overdue`
/// - `CreditFacilityRepaymentPlanJob` - rebuilds the repayment plan projection
/// - Dagster dbt pipeline - `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `id`: Unique obligation identifier
/// - `beneficiary_id`: Beneficiary identifier
/// - `outstanding_amount`: Current amount owed
/// - `overdue_at`: Effective overdue date
#[tokio::test]
async fn obligation_overdue_event_on_overdue_job() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    // The due/overdue/defaulted jobs run via the poller.
    ctx.jobs.start_poll().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let today = ctx.clock.today();
    // Set due in the past and overdue today to trigger the overdue job without advancing time.
    let due_date: EffectiveDate = today
        .checked_sub_days(chrono::Days::new(1))
        .expect("due date underflow")
        .into();
    let overdue_date: EffectiveDate = today.into();

    let (obligation, recorded) = expect_event(
        &ctx.outbox,
        || {
            create_obligation_with_dates(
                &ctx,
                beneficiary_id,
                amount,
                due_date,
                Some(overdue_date),
                None,
            )
        },
        |result, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationOverdue {
                entity,
            }) if entity.id == result.id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.initial_amount, amount);
    assert_eq!(recorded.outstanding_amount, amount);
    assert_eq!(recorded.due_at, due_date);
    assert_eq!(recorded.overdue_at, Some(overdue_date));
    assert!(recorded.defaulted_at.is_none());
    assert_eq!(recorded.id, obligation.id);

    ctx.jobs.shutdown().await?;
    Ok(())
}

/// `ObligationDefaulted` is published when an obligation moves to the defaulted state.
///
/// # Trigger
/// `ObligationDefaultedJobRunner::record_defaulted` (scheduled by `ObligationOverdueJobRunner::record_overdue`)
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` - marks repayment entry as `Defaulted`
/// - `CreditFacilityRepaymentPlanJob` - rebuilds the repayment plan projection
/// - Dagster dbt pipeline - `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `id`: Unique obligation identifier
/// - `beneficiary_id`: Beneficiary identifier
/// - `outstanding_amount`: Current amount owed
/// - `defaulted_at`: Effective defaulted date
#[tokio::test]
async fn obligation_defaulted_event_on_defaulted_job() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    // The due/overdue/defaulted jobs run via the poller.
    ctx.jobs.start_poll().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let today = ctx.clock.today();
    // Set due in the past and overdue/defaulted today to trigger the defaulted job immediately.
    let due_date: EffectiveDate = today
        .checked_sub_days(chrono::Days::new(1))
        .expect("due date underflow")
        .into();
    let overdue_date: EffectiveDate = today.into();
    let defaulted_date: EffectiveDate = today.into();

    let (obligation, recorded) = expect_event(
        &ctx.outbox,
        || {
            create_obligation_with_dates(
                &ctx,
                beneficiary_id,
                amount,
                due_date,
                Some(overdue_date),
                Some(defaulted_date),
            )
        },
        |result, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationDefaulted {
                entity,
            }) if entity.id == result.id => Some(entity.clone()),
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, beneficiary_id);
    assert_eq!(recorded.initial_amount, amount);
    assert_eq!(recorded.outstanding_amount, amount);
    assert_eq!(recorded.due_at, due_date);
    assert_eq!(recorded.overdue_at, Some(overdue_date));
    assert_eq!(recorded.defaulted_at, Some(defaulted_date));
    assert_eq!(recorded.id, obligation.id);

    ctx.jobs.shutdown().await?;
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
            &DummySubject::system(),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("payment was not created"))?;

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
                .allocate_payment_in_op(&mut op, payment_details, &DummySubject::system())
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
