mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use rust_decimal_macros::dec;

use core_credit::*;
use core_credit_collection::{CollectionLedgerError, PaymentError};
use core_deposit::DepositAccountId;
use document_storage::DocumentStorage;
use helpers::{action, event, object};
use public_id::PublicIds;

use core_credit::error::CoreCreditError;
use money::{Satoshis, UsdCents};

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
        .obligation_overdue_duration_from_due(ObligationDuration::Days(50))
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

type TestPerms = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;
type TestEvent = event::DummyEvent;

/// Creates a customer, deposit account, and activates a credit facility.
async fn create_active_facility(
    credit: &CoreCredit<TestPerms, TestEvent>,
    deposit: &core_deposit::CoreDeposit<TestPerms, TestEvent>,
    customers: &core_customer::Customers<TestPerms, TestEvent>,
    facility_amount: UsdCents,
) -> anyhow::Result<ActiveFacility> {
    // Create a customer
    let customer = customers
        .create_customer_bypassing_kyc(
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

    // Wait for governance approval (max 10 seconds)
    let max_retries = 100;
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
            panic!("Timed out waiting for governance approval after 10 seconds");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Update collateral to meet CVL requirements
    let pending_facility_id: PendingCreditFacilityId = proposal.id.into();
    let pending_facility = credit
        .pending_credit_facilities()
        .find_by_id(&DummySubject, pending_facility_id)
        .await?
        .expect("pending facility exists");
    let collateral_satoshis = Satoshis::from(50_000_000); // 0.5 BTC
    credit
        .collaterals()
        .update_collateral_by_id(
            &DummySubject,
            pending_facility.collateral_id,
            collateral_satoshis,
            chrono::Utc::now().date_naive(),
        )
        .await?;

    // Wait for facility activation (max 10 seconds)
    let facility_id: CreditFacilityId = proposal.id.into();
    let max_retries = 100;
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
            panic!("Timed out waiting for facility activation after 10 seconds");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(ActiveFacility {
        facility_id,
        deposit_account_id,
    })
}

/// Test that attempting to pay more than the outstanding obligations returns
/// the `PaymentAmountGreaterThanOutstandingObligations` error.
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn payment_exceeding_obligations_returns_error() -> anyhow::Result<()> {
    // Infrastructure setup
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .build()
            .expect("Couldn't build MailboxConfig"),
    )
    .await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let governance = governance::Governance::new(&pool, &authz, &outbox, clock.clone());
    let public_ids = public_id::PublicIds::new(&pool);
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
    // Required to prevent the case there is an attempt to remove an account set member from
    // an account set that no longer exists.
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
    jobs.start_poll().await?;

    // Create active facility
    let facility_amount = UsdCents::from(100_000); // $1,000
    let ActiveFacility {
        facility_id,
        deposit_account_id,
    } = create_active_facility(&credit, &deposit, &customers, facility_amount).await?;

    // Attempt overpayment and verify error
    let payment_amount = UsdCents::from(100);
    deposit
        .record_deposit(&DummySubject, deposit_account_id, payment_amount, None)
        .await?;
    let result = credit
        .record_payment(
            &DummySubject,
            facility_id,
            PaymentSourceAccountId::new(deposit_account_id.into()),
            payment_amount,
        )
        .await;
    assert!(result.is_ok());

    let payment_amount = facility_amount;
    deposit
        .record_deposit(&DummySubject, deposit_account_id, payment_amount, None)
        .await?;
    let result = credit
        .record_payment(
            &DummySubject,
            facility_id,
            PaymentSourceAccountId::new(deposit_account_id.into()),
            payment_amount,
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

    jobs.shutdown().await?;
    Ok(())
}
