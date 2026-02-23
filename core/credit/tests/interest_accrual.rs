mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use core_credit::*;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockController, ClockHandle};
use futures::StreamExt;
use money::{Satoshis, UsdCents};
use public_id::PublicIds;
use rust_decimal_macros::dec;
use std::time::Duration;

const ONE_DAY: Duration = Duration::from_secs(86400);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

async fn setup_with_clock_control()
-> anyhow::Result<(helpers::TestContext, ClockController, sqlx::PgPool)> {
    let pool = helpers::init_pool().await?;
    cleanup_stale_task_jobs(&pool).await?;
    let (clock, clock_ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<helpers::event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder().build()?,
    )
    .await?;

    let authz = authz::dummy::DummyPerms::<
        helpers::action::DummyAction,
        helpers::object::DummyObject,
    >::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let governance = governance::Governance::new(&pool, &authz, &outbox, clock.clone());
    let public_ids = PublicIds::new(&pool);
    let customers = core_customer::Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage,
        public_ids,
        clock.clone(),
    );

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .clock(clock.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let custody = core_custody::CoreCustody::init(
        &pool,
        &authz,
        helpers::custody_encryption_config(),
        helpers::custody_config(),
        &outbox,
        &mut jobs,
        clock.clone(),
    )
    .await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let journal_id = helpers::init_journal(&cala).await?;
    let credit_public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(
        core_price::PriceConfig { providers: vec![] },
        &mut jobs,
        &outbox,
    )
    .await?;
    let domain_configs = helpers::init_read_only_exposed_domain_configs(&pool, &authz).await?;
    // Reset this event-sourced config so each test builds a fresh
    // chart-of-accounts mapping. Reusing stale config can reference
    // account-set IDs that no longer exist in a clean integration run.
    domain_config::DomainConfigTestUtils::clear_config_by_key(
        &pool,
        "credit-chart-of-accounts-integration",
    )
    .await?;
    let internal_domain_configs = helpers::init_internal_domain_configs(&pool).await?;

    let credit = CoreCredit::init(
        &pool,
        &governance,
        &mut jobs,
        &authz,
        &customers,
        &custody,
        &price,
        &outbox,
        &cala,
        journal_id,
        &credit_public_ids,
        &domain_configs,
        &internal_domain_configs,
    )
    .await?;

    let deposit_public_ids = PublicIds::new(&pool);
    let deposit = core_deposit::CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &mut jobs,
        &cala,
        journal_id,
        &deposit_public_ids,
        &customers,
        &domain_configs,
        &internal_domain_configs,
    )
    .await?;

    helpers::seed_price(&outbox, &price).await?;

    Ok((
        helpers::TestContext {
            credit,
            deposit,
            customers,
            outbox,
            jobs,
        },
        clock_ctrl,
        pool,
    ))
}

fn daily_cycle_terms() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(3))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
        .obligation_liquidation_duration_from_due(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfDay)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::SingleDisbursal)
        .build()
        .unwrap()
}

/// Creates and activates a facility while manually advancing the artificial
/// clock in small steps so governance/activation jobs become due deterministically.
async fn create_active_facility_with_clock(
    ctx: &helpers::TestContext,
    clock_ctrl: &ClockController,
    terms: TermValues,
) -> anyhow::Result<helpers::ActiveFacilityState> {
    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("test-{}@example.com", uuid::Uuid::new_v4()),
            format!("telegram-{}", uuid::Uuid::new_v4()),
            core_customer::CustomerType::Individual,
        )
        .await?;

    let deposit_account = ctx
        .deposit
        .create_account(&DummySubject, customer.id)
        .await?;

    let amount = UsdCents::from(1_000_000);
    let proposal = ctx
        .credit
        .create_facility_proposal(
            &DummySubject,
            customer.id,
            deposit_account.id,
            amount,
            terms,
            None::<core_custody::CustodianId>,
        )
        .await?;

    ctx.credit
        .proposals()
        .conclude_customer_approval(&DummySubject, proposal.id, true)
        .await?;

    let pending_facility_id: PendingCreditFacilityId = proposal.id.into();

    // Wait for governance approval → pending facility creation.
    // Nudge the clock by 1 second per iteration so scheduled jobs can fire
    // without jumping dates far into the future.
    let pending_facility = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Some(pf) = ctx
                .credit
                .pending_credit_facilities()
                .find_by_id(&DummySubject, pending_facility_id)
                .await?
            {
                return Ok::<_, anyhow::Error>(pf);
            }
            clock_ctrl.advance(Duration::from_secs(1)).await;
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await
    .expect("Timed out waiting for governance approval")?;

    let collateral_satoshis = Satoshis::from(50_000_000);
    let effective = chrono::Utc::now().date_naive();
    let facility_id: CreditFacilityId = pending_facility_id.into();

    ctx.credit
        .collaterals()
        .update_collateral_by_id(
            &DummySubject,
            pending_facility.collateral_id,
            collateral_satoshis,
            effective,
        )
        .await?;

    // Wait for facility activation.
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Some(facility) = ctx
                .credit
                .facilities()
                .find_by_id(&DummySubject, facility_id)
                .await?
                && facility.status() == CreditFacilityStatus::Active
            {
                return Ok::<_, anyhow::Error>(());
            }
            clock_ctrl.advance(Duration::from_secs(1)).await;
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await
    .expect("Timed out waiting for facility activation")?;

    Ok(helpers::ActiveFacilityState {
        facility_id,
        collateral_id: pending_facility.collateral_id,
        deposit_account_id: deposit_account.id,
        customer_id: customer.id,
        amount,
    })
}

/// `AccrualPosted` is published when an interest accrual cycle completes.
///
/// # Trigger
/// `InterestAccrualJobRunner::complete_cycle`
/// (the final state in the AccruePeriod → AwaitObligationsSync → CompleteCycle machine)
///
/// # Consumers
/// - `History::process_credit_event` - records accrual posting
/// - `RepaymentPlan::process_credit_event` - updates projected repayment schedule
///
/// # Event Contents
/// - `id`: Interest accrual cycle identifier
/// - `credit_facility_id`: Parent facility
/// - `period`: The accrual period covered
/// - `due_at`: When the accrued interest becomes due
/// - `posting`: Ledger posting details (tx_id, amount, effective date)
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn accrual_posted_event_on_cycle_completion() -> anyhow::Result<()> {
    let (mut ctx, clock_ctrl, pool) = setup_with_clock_control().await?;
    ctx.jobs.start_poll().await?;

    let state = create_active_facility_with_clock(&ctx, &clock_ctrl, daily_cycle_terms()).await?;

    // Start listening BEFORE any clock advancement so we don't miss the event.
    let mut stream = ctx.outbox.listen_all(None);
    let facility_id = state.facility_id;

    // Advance the clock day-by-day while watching for AccrualPosted.
    // The disbursal balance appears quickly, then the interest accrual job
    // runs: AccruePeriod → AwaitObligationsSync → CompleteCycle → AccrualPosted
    let recorded = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    if let Some(CoreCreditEvent::AccrualPosted { entity }) =
                        event.as_event::<CoreCreditEvent>()
                        && entity.credit_facility_id == facility_id
                    {
                        return entity.clone();
                    }
                }
                _ = tokio::time::sleep(POLL_INTERVAL) => {
                    clock_ctrl.advance(ONE_DAY).await;
                }
            }
        }
    })
    .await
    .expect("Timed out waiting for AccrualPosted event");

    assert_eq!(recorded.credit_facility_id, state.facility_id);
    let posting = recorded
        .posting
        .expect("posting should be present after cycle completion");
    // 12% annual on $10,000 for 1 day: 10000 * 1 * 0.12 / 365 = 328.77 → 329 (rounded away from zero)
    assert_eq!(posting.amount, UsdCents::from(329));

    // `shutdown()` calls `kill_remaining_jobs`, which rewrites still-running
    // rows to `pending` with `execute_at = clock.now()`. Because this test
    // advances artificial time, those timestamps can end up ahead of wall-clock
    // time. Transition first so rewritten rows use real time and are immediately
    // eligible in subsequent realtime tests.
    clock_ctrl.transition_to_realtime();
    ctx.jobs.shutdown().await?;
    cleanup_stale_task_jobs(&pool).await?;
    Ok(())
}

async fn cleanup_stale_task_jobs(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "DELETE FROM job_executions
         WHERE state = 'pending'
           AND job_type IN ('task.interest-accrual', 'task.credit-facility-maturity')",
    )
    .execute(pool)
    .await?;
    Ok(())
}
