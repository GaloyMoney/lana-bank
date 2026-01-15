mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use cloud_storage::{Storage, config::StorageConfig};
use rust_decimal_macros::dec;

use core_credit::{ledger::error::CreditLedgerError, *};
use core_deposit::DepositAccountId;
use document_storage::DocumentStorage;
use helpers::{action, event, object};
use public_id::PublicIds;

use core_credit::error::CoreCreditError;
use core_money::{Satoshis, UsdCents};

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

    // Wait for governance approval
    loop {
        if let Some(prop) = credit
            .proposals()
            .find_by_id(&DummySubject, proposal.id)
            .await?
            && prop.status() == CreditFacilityProposalStatus::Approved
        {
            break;
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

    // Wait for facility activation
    let facility_id: CreditFacilityId = proposal.id.into();
    loop {
        if let Some(facility) = credit
            .facilities()
            .find_by_id(&DummySubject, facility_id)
            .await?
            && facility.status() == CreditFacilityStatus::Active
        {
            break;
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
async fn payment_exceeding_obligations_returns_error() -> anyhow::Result<()> {
    // Infrastructure setup
    let pool = helpers::init_pool().await?;
    let outbox =
        obix::Outbox::<event::DummyEvent>::init(&pool, obix::MailboxConfig::default()).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, &outbox);
    let governance = governance::Governance::new(&pool, &authz, &outbox);
    let public_ids = public_id::PublicIds::new(&pool);
    let customers =
        core_customer::Customers::new(&pool, &authz, &outbox, document_storage, public_ids);
    let custody =
        core_custody::CoreCustody::init(&pool, &authz, helpers::custody_config(), &outbox).await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .poller_config(job::JobPollerConfig {
                job_lost_interval: std::time::Duration::from_secs(2),
                ..Default::default()
            })
            .build()
            .unwrap(),
    )
    .await?;
    let mut jobs_new = job_new::Jobs::init(
        job_new::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;
    let journal_id = helpers::init_journal(&cala).await?;
    let credit_public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(&mut jobs_new, &outbox).await?;
    let credit = CoreCredit::init(
        &pool,
        CreditConfig {
            customer_active_check_enabled: false,
            ..Default::default()
        },
        &governance,
        &jobs,
        &authz,
        &customers,
        &custody,
        &price,
        &outbox,
        &cala,
        journal_id,
        &credit_public_ids,
    )
    .await?;
    let deposit_public_ids = PublicIds::new(&pool);
    let deposit = core_deposit::CoreDeposit::init(
        &pool,
        &authz,
        &outbox,
        &governance,
        &mut jobs_new,
        &cala,
        journal_id,
        &deposit_public_ids,
        &customers,
        core_deposit::DepositConfig {
            require_verified_customer_for_account: false,
        },
    )
    .await?;
    jobs.start_poll().await?;
    jobs_new.start_poll().await?;

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
                PaymentError::CreditLedgerError(
                    CreditLedgerError::PaymentAmountGreaterThanOutstandingObligations,
                )
            )),
        ),
        "{}",
        match &result {
            Err(e) => format!("{}", e),
            Ok(f) => format!("Credit Facility: {}", f.id),
        },
    );

    Ok(())
}
