mod helpers;

use audit::SystemSubject;
use authz::dummy::DummySubject;
use cala_ledger::primitives::TransactionId as LedgerTxId;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollectionEvent, NewObligation, Obligation, ObligationId,
    ObligationStatus, ObligationType, PaymentDetailsForAllocation, PaymentId,
    PaymentLedgerAccountIds, PaymentSourceAccountId,
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
        .create_in_op(&mut op, new_obligation)
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
/// `Obligations::create_in_op`
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
/// `Obligations::process_obligations_for_day` (triggered by EndOfDay event)
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
async fn obligation_due_event_on_process_day() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    // Due today so process_obligations_for_day transitions immediately.
    let due_date: EffectiveDate = ctx.clock.today().into();

    let obligation =
        create_obligation_with_dates(&ctx, beneficiary_id, amount, due_date, None, None).await?;

    let obligations = ctx.collections.obligations().clone();
    let day = ctx.clock.today();
    let obligation_id = obligation.id;

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            obligations.process_obligations_for_day(day).await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationDue {
                entity,
            }) if entity.id == obligation_id => Some(entity.clone()),
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

    Ok(())
}

/// `ObligationOverdue` is published when an obligation moves to the overdue state.
///
/// # Trigger
/// `Obligations::process_obligations_for_day` (triggered by EndOfDay event)
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
async fn obligation_overdue_event_on_process_day() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let today = ctx.clock.today();
    // Set due in the past and overdue today to trigger both transitions.
    let due_date: EffectiveDate = today
        .checked_sub_days(chrono::Days::new(1))
        .expect("due date underflow")
        .into();
    let overdue_date: EffectiveDate = today.into();

    let obligation = create_obligation_with_dates(
        &ctx,
        beneficiary_id,
        amount,
        due_date,
        Some(overdue_date),
        None,
    )
    .await?;

    let obligations = ctx.collections.obligations().clone();
    let day = ctx.clock.today();
    let obligation_id = obligation.id;

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            obligations.process_obligations_for_day(day).await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationOverdue {
                entity,
            }) if entity.id == obligation_id => Some(entity.clone()),
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

    Ok(())
}

/// `ObligationDefaulted` is published when an obligation moves to the defaulted state.
///
/// # Trigger
/// `Obligations::process_obligations_for_day` (triggered by EndOfDay event)
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
async fn obligation_defaulted_event_on_process_day() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;

    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let today = ctx.clock.today();
    // Set due in the past and overdue/defaulted today to trigger all transitions.
    let due_date: EffectiveDate = today
        .checked_sub_days(chrono::Days::new(1))
        .expect("due date underflow")
        .into();
    let overdue_date: EffectiveDate = today.into();
    let defaulted_date: EffectiveDate = today.into();

    let obligation = create_obligation_with_dates(
        &ctx,
        beneficiary_id,
        amount,
        due_date,
        Some(overdue_date),
        Some(defaulted_date),
    )
    .await?;

    let obligations = ctx.collections.obligations().clone();
    let day = ctx.clock.today();
    let obligation_id = obligation.id;

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            obligations.process_obligations_for_day(day).await?;
            Ok::<_, anyhow::Error>(())
        },
        move |_, e| match e {
            DummyEvent::CoreCreditCollection(CoreCreditCollectionEvent::ObligationDefaulted {
                entity,
            }) if entity.id == obligation_id => Some(entity.clone()),
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

    // Record a payment and allocate it in a single transaction to avoid
    // races with concurrent test processes that poll for PaymentCreated events.
    let payment_ledger_accounts = PaymentLedgerAccountIds {
        facility_payment_holding_account_id: ctx.accounts.payment_holding,
        facility_uncovered_outstanding_account_id: ctx.accounts.uncovered_outstanding,
        payment_source_account_id: PaymentSourceAccountId::new(ctx.accounts.payment_source),
    };

    let obligations = ctx.collections.obligations().clone();
    let pool = ctx.pool.clone();
    let clock = ctx.clock.clone();
    let effective = ctx.clock.today();
    let obligation_id = obligation.id;

    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || async move {
            let mut op = DbOp::init_with_clock(&pool, &clock).await?;
            let payment = ctx
                .collections
                .payments()
                .record_in_op(
                    &mut op,
                    PaymentId::new(),
                    beneficiary_id,
                    payment_ledger_accounts,
                    amount,
                    effective,
                    &DummySubject::system(audit::SystemActor::new("test")),
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("payment was not created"))?;
            let payment_details = PaymentDetailsForAllocation::from(payment);
            obligations
                .allocate_payment_in_op(
                    &mut op,
                    payment_details,
                    &DummySubject::system(audit::SystemActor::new("test")),
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

/// Calling process_obligations_for_day twice for the same day is idempotent.
#[tokio::test]
async fn process_obligations_for_day_is_idempotent() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let due_date: EffectiveDate = ctx.clock.today().into();

    create_obligation_with_dates(&ctx, beneficiary_id, amount, due_date, None, None).await?;

    let day = ctx.clock.today();
    ctx.collections
        .obligations()
        .process_obligations_for_day(day)
        .await?;

    // Calling again should not error
    ctx.collections
        .obligations()
        .process_obligations_for_day(day)
        .await?;

    Ok(())
}

/// Obligations with past due dates are processed when a later day is given (catch-up).
#[tokio::test]
async fn process_obligations_catches_up_past_due_dates() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    let today = ctx.clock.today();
    // due_date is 5 days in the past
    let due_date: EffectiveDate = today
        .checked_sub_days(chrono::Days::new(5))
        .expect("due date underflow")
        .into();

    let obligation =
        create_obligation_with_dates(&ctx, beneficiary_id, amount, due_date, None, None).await?;

    // Processing with today's date should catch up
    ctx.collections
        .obligations()
        .process_obligations_for_day(today)
        .await?;

    let updated = ctx
        .collections
        .obligations()
        .find_by_id_without_audit(obligation.id)
        .await?;
    assert_eq!(updated.status(), ObligationStatus::Due);

    Ok(())
}

/// Paid obligations are not transitioned by process_obligations_for_day.
#[tokio::test]
async fn paid_obligations_are_skipped() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;
    let beneficiary_id = BeneficiaryId::new();
    let amount = UsdCents::from(100_000);
    // Use a future due_date so the obligation is NotYetDue and won't be picked up
    // by other concurrent tests calling process_obligations_for_day(today).
    let due_date: EffectiveDate = ctx
        .clock
        .today()
        .checked_add_days(chrono::Days::new(30))
        .expect("due date overflow")
        .into();

    let obligation =
        create_obligation_with_dates(&ctx, beneficiary_id, amount, due_date, None, None).await?;

    // Pay off the obligation
    let payment_ledger_accounts = PaymentLedgerAccountIds {
        facility_payment_holding_account_id: ctx.accounts.payment_holding,
        facility_uncovered_outstanding_account_id: ctx.accounts.uncovered_outstanding,
        payment_source_account_id: PaymentSourceAccountId::new(ctx.accounts.payment_source),
    };
    let effective = ctx.clock.today();
    let mut op = DbOp::init_with_clock(&ctx.pool, &ctx.clock).await?;
    let payment = ctx
        .collections
        .payments()
        .record_in_op(
            &mut op,
            PaymentId::new(),
            beneficiary_id,
            payment_ledger_accounts,
            amount,
            effective,
            &DummySubject::system(audit::SystemActor::new("test")),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("payment was not created"))?;
    let payment_details = PaymentDetailsForAllocation::from(payment);
    ctx.collections
        .obligations()
        .allocate_payment_in_op(
            &mut op,
            payment_details,
            &DummySubject::system(audit::SystemActor::new("test")),
        )
        .await?;
    op.commit().await?;

    let updated = ctx
        .collections
        .obligations()
        .find_by_id_without_audit(obligation.id)
        .await?;
    assert_eq!(updated.status(), ObligationStatus::Paid);

    // Process day with due_date — paid obligation should not transition even though
    // the date matches.
    let day = chrono::NaiveDate::from(due_date);
    ctx.collections
        .obligations()
        .process_obligations_for_day(day)
        .await?;

    let after = ctx
        .collections
        .obligations()
        .find_by_id_without_audit(obligation.id)
        .await?;
    assert_eq!(after.status(), ObligationStatus::Paid);

    Ok(())
}
