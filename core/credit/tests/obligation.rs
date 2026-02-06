mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use obix::test_utils::expect_event;
use public_id::PublicIds;
use rust_decimal_macros::dec;

use core_credit::*;
use core_credit_collection::CoreCreditCollectionEvent;
use core_deposit::DepositAccountId;
use core_money::{Satoshis, UsdCents};
use helpers::{action, event, object};

type TestPerms = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;
type TestEvent = event::DummyEvent;

async fn setup() -> anyhow::Result<(
    CoreCredit<TestPerms, TestEvent>,
    core_deposit::CoreDeposit<TestPerms, TestEvent>,
    core_customer::Customers<TestPerms, TestEvent>,
    obix::Outbox<TestEvent>,
    job::Jobs,
)> {
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
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
            .build()
            .unwrap(),
    )
    .await?;

    let custody = core_custody::CoreCustody::init(
        &pool,
        &authz,
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

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let journal_id = helpers::init_journal(&cala).await?;
    let credit_public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(&mut jobs, &outbox).await?;
    let domain_configs = helpers::init_read_only_exposed_domain_configs(&pool, &authz).await?;
    helpers::clear_internal_domain_config(&pool, "credit-chart-of-accounts-integration").await?;
    let internal_domain_configs = helpers::init_internal_domain_configs(&pool).await?;

    let credit = CoreCredit::init(
        &pool,
        CreditConfig::default(),
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

    jobs.start_poll().await?;

    helpers::seed_price(
        &outbox,
        core_price::PriceOfOneBTC::new(UsdCents::from(5_000_000)),
    )
    .await?;

    Ok((credit, deposit, customers, outbox, jobs))
}

fn random_email() -> String {
    format!("{}@integrationtest.com", uuid::Uuid::new_v4())
}

fn random_username() -> String {
    format!("{}", uuid::Uuid::new_v4())
}

fn test_term_values() -> TermValues {
    TermValues::builder()
        .annual_rate(dec!(12))
        .initial_cvl(dec!(140))
        .margin_call_cvl(dec!(125))
        .liquidation_cvl(dec!(105))
        .duration(FacilityDuration::Months(3))
        .interest_due_duration_from_accrual(ObligationDuration::Days(0))
        .obligation_overdue_duration_from_due(ObligationDuration::Days(30))
        .obligation_liquidation_duration_from_due(None)
        .accrual_interval(InterestInterval::EndOfDay)
        .accrual_cycle_interval(InterestInterval::EndOfMonth)
        .one_time_fee_rate(dec!(0.01))
        .disbursal_policy(DisbursalPolicy::SingleDisbursal)
        .build()
        .unwrap()
}

struct ActiveFacility {
    facility_id: CreditFacilityId,
    deposit_account_id: DepositAccountId,
}

/// Creates a customer, deposit account, and activates a credit facility.
async fn create_active_facility(
    credit: &CoreCredit<TestPerms, TestEvent>,
    deposit: &core_deposit::CoreDeposit<TestPerms, TestEvent>,
    customers: &core_customer::Customers<TestPerms, TestEvent>,
    facility_amount: UsdCents,
) -> anyhow::Result<ActiveFacility> {
    // Create a customer
    let customer = customers
        .create(
            &DummySubject,
            random_email(),
            random_username(),
            core_customer::CustomerType::Individual,
        )
        .await?;

    // Create deposit account
    let deposit_account = deposit.create_account(&DummySubject, customer.id).await?;
    let deposit_account_id = deposit_account.id;

    // Create facility proposal
    let proposal = credit
        .create_facility_proposal(
            &DummySubject,
            customer.id,
            deposit_account_id,
            facility_amount,
            test_term_values(),
            None::<core_custody::CustodianId>,
        )
        .await?;

    // Customer approval (triggers governance)
    credit
        .proposals()
        .conclude_customer_approval(&DummySubject, proposal.id, true)
        .await?;

    // Wait for governance approval (max 20 seconds)
    let max_retries = 200;
    for attempt in 0..max_retries {
        if let Some(prop) = credit
            .proposals()
            .find_by_id(&DummySubject, proposal.id)
            .await?
            && prop.status() == CreditFacilityProposalStatus::Approved
        {
            break;
        }
        if attempt == max_retries - 1 {
            panic!("Timed out waiting for governance approval after 20 seconds");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Update collateral to meet CVL requirements
    let pending_facility_id: PendingCreditFacilityId = proposal.id.into();
    let collateral_satoshis = Satoshis::from(50_000_000); // 0.5 BTC
    credit
        .update_pending_facility_collateral(
            &DummySubject,
            pending_facility_id,
            collateral_satoshis,
            chrono::Utc::now().date_naive(),
        )
        .await?;

    // Wait for facility activation (max 20 seconds)
    let facility_id: CreditFacilityId = proposal.id.into();
    let max_retries = 200;
    for attempt in 0..max_retries {
        if let Some(facility) = credit
            .facilities()
            .find_by_id(&DummySubject, facility_id)
            .await?
            && facility.status() == CreditFacilityStatus::Active
        {
            break;
        }
        if attempt == max_retries - 1 {
            panic!("Timed out waiting for facility activation after 20 seconds");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(ActiveFacility {
        facility_id,
        deposit_account_id,
    })
}

/// `ObligationCreated` is published when a new obligation is created.
///
/// # Trigger
/// Disbursal is approved via the governance approval process.
/// create_active_facility (test helper)
///   → update_pending_facility_collateral   
///     → [async job] activate_credit_facility
///       → CreditFacilities::activate
///         → Disbursals::create_pre_approved_disbursal_in_op
///           → Disbursal::approval_process_concluded_for_initial_disbursal
///             → settle_disbursal → NewObligation(Disbursal)
///               → Obligations::create_with_jobs_in_op
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` — adds a new repayment entry
/// - `CreditFacilityRepaymentPlanJob` — triggers repayment plan rebuild
/// - `collateralization_from_events` — updates collateralization state
/// - Dagster dbt pipeline — `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `id`: Unique obligation identifier
/// - `obligation_type`: `Disbursal` or `Interest`
/// - `beneficiary_id`: Credit facility ID
/// - `initial_amount`: Original obligation amount
/// - `outstanding_amount`: Current amount owed
/// - `due_at`, `overdue_at`, `defaulted_at`: Lifecycle dates
#[tokio::test]
async fn obligation_created_event_on_disbursal_approval() -> anyhow::Result<()> {
    let (credit, deposit, customers, outbox, jobs) = setup().await?;

    let facility_amount = UsdCents::from(100_000);

    let (ActiveFacility { facility_id, .. }, recorded) = expect_event(
        &outbox,
        || create_active_facility(&credit, &deposit, &customers, facility_amount),
        |result, e| match e {
            CoreCreditCollectionEvent::ObligationCreated { entity }
                if entity.beneficiary_id == result.facility_id.into() =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, facility_id.into());
    assert_eq!(
        recorded.obligation_type,
        core_credit_collection::ObligationType::Disbursal
    );
    assert_eq!(recorded.initial_amount, facility_amount);
    assert_eq!(recorded.outstanding_amount, facility_amount);

    jobs.shutdown().await?;
    Ok(())
}

/// `ObligationCompleted` is published when an obligation is fully paid off.
///
/// # Trigger
/// Payment allocation reduces outstanding amount to zero.
/// CoreCredit::record_payment
///   → [async job] allocate_credit_facility_payment
///     → Obligations::allocate_payment_in_op
///       → Obligation::allocate_payment
///         → outstanding().is_zero() → ObligationEvent::Completed
///
/// # Consumers
/// - `RepaymentPlan::process_collection_event` — marks repayment entry as `Paid`
/// - `CreditFacilityRepaymentPlanJob` — triggers repayment plan rebuild
/// - Dagster dbt pipeline — `int_core_obligation_events_rollup_sequence.sql`
///
/// # Event Contents
/// - `outstanding_amount`: Zero
/// - All other fields from `ObligationCreated`
#[tokio::test]
async fn obligation_completed_event_on_full_payment() -> anyhow::Result<()> {
    let (credit, deposit, customers, outbox, jobs) = setup().await?;

    let facility_amount = UsdCents::from(100_000);
    let ActiveFacility {
        facility_id,
        deposit_account_id,
    } = create_active_facility(&credit, &deposit, &customers, facility_amount).await?;

    deposit
        .record_deposit(&DummySubject, deposit_account_id, facility_amount, None)
        .await?;

    let (_, recorded) = expect_event(
        &outbox,
        || {
            credit.record_payment(
                &DummySubject,
                facility_id,
                PaymentSourceAccountId::new(deposit_account_id.into()),
                facility_amount,
            )
        },
        |_, e| match e {
            CoreCreditCollectionEvent::ObligationCompleted { entity }
                if entity.beneficiary_id == facility_id.into() =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.beneficiary_id, facility_id.into());
    assert_eq!(recorded.outstanding_amount, UsdCents::ZERO);
    assert_eq!(recorded.initial_amount, facility_amount);

    jobs.shutdown().await?;
    Ok(())
}
