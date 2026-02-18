#![allow(dead_code)] // Helper functions may not be used in all tests

use std::collections::HashMap;

use authz::dummy::DummySubject;
use cala_ledger::CalaLedgerConfig;
use cala_ledger::account_set::AccountSetMemberId;
use cala_ledger::{CalaLedger, JournalId};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::{AccountCode, AccountingBaseConfig, CalaAccountSetId, Chart, CoreAccounting};
use core_credit::*;
use core_credit::{CreditOmnibusAccountSetSpec, CreditSummaryAccountSetSpec};
use core_custody::{CustodyConfig, EncryptionConfig};
use document_storage::DocumentStorage;
use domain_config::{
    EncryptionConfig as DomainEncryptionConfig, ExposedDomainConfigs, ExposedDomainConfigsReadOnly,
    InternalDomainConfigs, RequireVerifiedCustomerForAccount,
};
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use money::Satoshis;
use public_id::PublicIds;
use rand::Rng;
use rust_decimal_macros::dec;
use std::time::Duration;
pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_read_only_exposed_domain_configs(
    pool: &sqlx::PgPool,
    authz: &authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
) -> anyhow::Result<ExposedDomainConfigsReadOnly> {
    let exposed_configs = ExposedDomainConfigs::new(pool, authz, DomainEncryptionConfig::default());
    exposed_configs.seed_registered().await?;
    // Disable the require verified customer check for tests
    // Ignore concurrent modification - all tests want the same value (false)
    let _ = exposed_configs
        .update::<RequireVerifiedCustomerForAccount>(&authz::dummy::DummySubject, false)
        .await;
    Ok(ExposedDomainConfigsReadOnly::new(
        pool,
        DomainEncryptionConfig::default(),
    ))
}

pub async fn init_internal_domain_configs(
    pool: &sqlx::PgPool,
) -> anyhow::Result<InternalDomainConfigs> {
    let internal_configs = InternalDomainConfigs::new(pool, DomainEncryptionConfig::default());
    internal_configs.seed_registered().await?;
    Ok(internal_configs)
}

pub fn test_btc_price() -> core_price::PriceOfOneBTC {
    core_price::PriceOfOneBTC::new(money::UsdCents::from(7_000_000))
}

pub async fn seed_price<E>(
    outbox: &obix::Outbox<E>,
    price: &core_price::Price,
) -> anyhow::Result<core_price::PriceOfOneBTC>
where
    E: obix::out::OutboxEventMarker<core_price::CorePriceEvent> + Send + Sync + 'static,
{
    let seeded_price = test_btc_price();
    outbox
        .publish_ephemeral(
            core_price::PRICE_UPDATED_EVENT_TYPE,
            core_price::CorePriceEvent::PriceUpdated {
                price: seeded_price,
                timestamp: chrono::Utc::now(),
            },
        )
        .await?;

    tokio::time::timeout(std::time::Duration::from_secs(5), price.usd_cents_per_btc())
        .await
        .map_err(|_| anyhow::anyhow!("Timed out waiting for test BTC price to propagate"))?;

    Ok(seeded_price)
}

pub fn custody_config() -> CustodyConfig {
    CustodyConfig {
        encryption: EncryptionConfig {
            key: [1u8; 32].into(),
        },
        deprecated_encryption_key: None,
        custody_providers: Default::default(),
    }
}

pub async fn init_journal(cala: &CalaLedger) -> anyhow::Result<cala_ledger::JournalId> {
    use cala_ledger::journal::*;

    let id = JournalId::new();
    let new = NewJournal::builder()
        .id(id)
        .name("Test journal")
        .build()
        .unwrap();
    let journal = cala.journals().create(new).await?;
    Ok(journal.id)
}

pub fn default_accounting_base_config() -> AccountingBaseConfig {
    AccountingBaseConfig::try_new(
        "1".parse().unwrap(),
        "2".parse().unwrap(),
        "3".parse().unwrap(),
        "32.01".parse().unwrap(),
        "32.02".parse().unwrap(),
        "4".parse().unwrap(),
        "5".parse().unwrap(),
        "6".parse().unwrap(),
    )
    .unwrap()
}

pub const BASE_ACCOUNTS_CSV: &str = r#"
1,,,Assets,Debit,
2,,,Liabilities,Credit,
3,,,Equity,Credit,
32,,,Retained Earnings,,
,01,,Annual Gains,,
,02,,Annual Losses,,
4,,,Revenue,Credit,
5,,,Cost of Revenue,Debit,
6,,,Expenses,Debit,
8,,,Off Balance Sheet,Credit,
"#;

pub fn chart_account_set_id(chart: &Chart, code: &AccountCode) -> CalaAccountSetId {
    chart
        .maybe_account_set_id_from_code(code)
        .unwrap_or_else(|| panic!("missing account set for code"))
}

pub async fn assert_attached_for_code(
    cala: &CalaLedger,
    chart: &Chart,
    code: &AccountCode,
    expected_child_id: CalaAccountSetId,
) -> anyhow::Result<()> {
    assert_account_sets_attached(
        cala,
        chart_account_set_id(chart, code),
        &[expected_child_id],
    )
    .await
}

fn ledger_external_ref(journal_id: JournalId, external_ref: &str) -> String {
    let mut reference = journal_id.to_string();
    reference.push(':');
    reference.push_str(external_ref);
    reference
}

async fn account_set_id_by_ref(
    cala: &CalaLedger,
    journal_id: JournalId,
    external_ref: &str,
) -> anyhow::Result<CalaAccountSetId> {
    Ok(cala
        .account_sets()
        .find_by_external_id(ledger_external_ref(journal_id, external_ref))
        .await?
        .id)
}

async fn account_exists(
    cala: &CalaLedger,
    journal_id: JournalId,
    external_ref: &str,
) -> anyhow::Result<()> {
    cala.accounts()
        .find_by_external_id(ledger_external_ref(journal_id, external_ref))
        .await?;
    Ok(())
}

pub async fn resolve_account_set_ids<I>(
    cala: &CalaLedger,
    journal_id: JournalId,
    specs: I,
) -> anyhow::Result<HashMap<&'static str, CalaAccountSetId>>
where
    I: IntoIterator<Item = CreditSummaryAccountSetSpec>,
{
    let mut ids = HashMap::new();
    for spec in specs {
        let id = account_set_id_by_ref(cala, journal_id, spec.external_ref).await?;
        ids.insert(spec.external_ref, id);
    }
    Ok(ids)
}

pub async fn resolve_omnibus_account_set_ids<I>(
    cala: &CalaLedger,
    journal_id: JournalId,
    specs: I,
) -> anyhow::Result<HashMap<&'static str, CalaAccountSetId>>
where
    I: IntoIterator<Item = CreditOmnibusAccountSetSpec>,
{
    let mut ids = HashMap::new();
    for spec in specs {
        let id = account_set_id_by_ref(cala, journal_id, spec.account_set_ref).await?;
        account_exists(cala, journal_id, spec.account_ref).await?;
        ids.insert(spec.account_set_ref, id);
    }
    Ok(ids)
}

async fn assert_account_sets_attached(
    cala: &CalaLedger,
    parent_id: CalaAccountSetId,
    expected_child_ids: &[CalaAccountSetId],
) -> anyhow::Result<()> {
    let members = cala
        .account_sets()
        .list_members_by_created_at(parent_id, Default::default())
        .await?
        .entities;

    for expected_id in expected_child_ids {
        assert!(
            members
                .iter()
                .any(|member| member.id == AccountSetMemberId::AccountSet(*expected_id)),
            "expected account set to be attached to parent account set",
        );
    }

    Ok(())
}

pub async fn create_test_statements<Perms, E>(
    accounting: &CoreAccounting<Perms, E>,
) -> anyhow::Result<(String, String, String)>
where
    Perms: authz::PermissionCheck,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_accounting::CoreAccountingAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_accounting::CoreAccountingObject>,
    E: obix::out::OutboxEventMarker<core_accounting::CoreAccountingEvent>,
{
    let bs = format!("BS-{:010}", rand::rng().random_range(0..10_000_000_000u64));
    let pl = format!("PL-{:010}", rand::rng().random_range(0..10_000_000_000u64));
    let tb = format!("TB-{:010}", rand::rng().random_range(0..10_000_000_000u64));

    accounting
        .balance_sheets()
        .create_balance_sheet(bs.clone())
        .await?;
    accounting
        .profit_and_loss()
        .create_pl_statement(pl.clone())
        .await?;
    accounting
        .trial_balances()
        .create_trial_balance_statement(tb.clone())
        .await?;

    Ok((bs, pl, tb))
}

pub mod action {
    use core_accounting::CoreAccountingAction;
    use core_credit::CoreCreditAction;
    use core_credit::CoreCreditCollectionAction;
    use core_custody::CoreCustodyAction;
    use core_customer::CoreCustomerAction;
    use core_deposit::CoreDepositAction;
    use domain_config::DomainConfigAction;
    use governance::GovernanceAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<CoreCreditAction> for DummyAction {
        fn from(_: CoreCreditAction) -> Self {
            Self
        }
    }

    impl From<CoreCreditCollectionAction> for DummyAction {
        fn from(_: CoreCreditCollectionAction) -> Self {
            Self
        }
    }

    impl From<GovernanceAction> for DummyAction {
        fn from(_: GovernanceAction) -> Self {
            Self
        }
    }

    impl From<CoreCustomerAction> for DummyAction {
        fn from(_: CoreCustomerAction) -> Self {
            Self
        }
    }

    impl From<CoreCustodyAction> for DummyAction {
        fn from(_: CoreCustodyAction) -> Self {
            Self
        }
    }

    impl From<CoreAccountingAction> for DummyAction {
        fn from(_: CoreAccountingAction) -> Self {
            Self
        }
    }

    impl From<CoreDepositAction> for DummyAction {
        fn from(_: CoreDepositAction) -> Self {
            Self
        }
    }

    impl From<DomainConfigAction> for DummyAction {
        fn from(_: DomainConfigAction) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")?;
            Ok(())
        }
    }

    impl std::str::FromStr for DummyAction {
        type Err = strum::ParseError;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self)
        }
    }
}

pub mod object {
    use core_accounting::CoreAccountingObject;
    use core_credit::CoreCreditCollectionObject;
    use core_credit::CoreCreditObject;
    use core_custody::CoreCustodyObject;
    use core_customer::CustomerObject;
    use core_deposit::CoreDepositObject;
    use domain_config::DomainConfigObject;
    use governance::GovernanceObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreCreditObject> for DummyObject {
        fn from(_: CoreCreditObject) -> Self {
            Self
        }
    }
    impl From<CoreCreditCollectionObject> for DummyObject {
        fn from(_: CoreCreditCollectionObject) -> Self {
            Self
        }
    }
    impl From<CoreAccountingObject> for DummyObject {
        fn from(_: CoreAccountingObject) -> Self {
            Self
        }
    }

    impl From<GovernanceObject> for DummyObject {
        fn from(_: GovernanceObject) -> Self {
            Self
        }
    }

    impl From<CustomerObject> for DummyObject {
        fn from(_: CustomerObject) -> Self {
            Self
        }
    }

    impl From<CoreCustodyObject> for DummyObject {
        fn from(_: CoreCustodyObject) -> Self {
            Self
        }
    }

    impl From<CoreDepositObject> for DummyObject {
        fn from(_: CoreDepositObject) -> Self {
            Self
        }
    }

    impl From<DomainConfigObject> for DummyObject {
        fn from(_: DomainConfigObject) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Dummy")?;
            Ok(())
        }
    }

    impl std::str::FromStr for DummyObject {
        type Err = &'static str;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(DummyObject)
        }
    }
}

pub mod event {
    use serde::{Deserialize, Serialize};

    use core_access::CoreAccessEvent;
    use core_accounting::CoreAccountingEvent;
    use core_credit::CoreCreditCollectionEvent;
    use core_credit::CoreCreditEvent;
    use core_custody::CoreCustodyEvent;
    use core_customer::CoreCustomerEvent;
    use core_deposit::CoreDepositEvent;
    use core_price::CorePriceEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreAccess(CoreAccessEvent),
        CoreAccounting(CoreAccountingEvent),
        CoreCredit(CoreCreditEvent),
        CoreCreditCollection(CoreCreditCollectionEvent),
        CoreCustody(CoreCustodyEvent),
        CoreCustomer(CoreCustomerEvent),
        CoreDeposit(CoreDepositEvent),
        Governance(GovernanceEvent),
        Price(CorePriceEvent),
        #[serde(other)]
        Unknown,
    }

    #[allow(unused_imports)]
    pub use obix::test_utils::expect_event;
}

pub type TestPerms = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;
pub type TestEvent = event::DummyEvent;

pub struct TestContext {
    pub credit: CoreCredit<TestPerms, TestEvent>,
    pub deposit: core_deposit::CoreDeposit<TestPerms, TestEvent>,
    pub customers: core_customer::Customers<TestPerms, TestEvent>,
    pub outbox: obix::Outbox<TestEvent>,
    pub jobs: job::Jobs,
}

pub fn test_terms() -> TermValues {
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

pub async fn setup() -> anyhow::Result<TestContext> {
    let pool = init_pool().await?;
    let (clock, _ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox =
        obix::Outbox::<TestEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let authz = TestPerms::new();
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
        custody_config(),
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

    let journal_id = init_journal(&cala).await?;
    let credit_public_ids = PublicIds::new(&pool);
    let price = core_price::Price::init(&mut jobs, &outbox).await?;
    let domain_configs = init_read_only_exposed_domain_configs(&pool, &authz).await?;
    domain_config::DomainConfigTestUtils::clear_config_by_key(
        &pool,
        "credit-chart-of-accounts-integration",
    )
    .await?;
    let internal_domain_configs = init_internal_domain_configs(&pool).await?;

    let credit = CoreCredit::init(
        &pool,
        Default::default(),
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

    seed_price(&outbox, &price).await?;

    Ok(TestContext {
        credit,
        deposit,
        customers,
        outbox,
        jobs,
    })
}

pub struct PendingFacilityState {
    pub customer_id: CustomerId,
    pub pending_facility_id: PendingCreditFacilityId,
    pub collateral_id: CollateralId,
    pub deposit_account_id: core_deposit::DepositAccountId,
    pub amount: money::UsdCents,
    pub terms: TermValues,
}

pub async fn create_pending_facility(
    ctx: &TestContext,
    terms: TermValues,
) -> anyhow::Result<PendingFacilityState> {
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

    let amount = money::UsdCents::from(1_000_000);
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
    for attempt in 0..100 {
        if let Some(pf) = ctx
            .credit
            .pending_credit_facilities()
            .find_by_id(&DummySubject, pending_facility_id)
            .await?
        {
            return Ok(PendingFacilityState {
                customer_id: customer.id,
                pending_facility_id,
                collateral_id: pf.collateral_id,
                deposit_account_id: deposit_account.id,
                amount,
                terms,
            });
        }
        if attempt == 99 {
            panic!("Timed out waiting for pending facility creation");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    unreachable!()
}

pub struct ActiveFacilityState {
    pub facility_id: CreditFacilityId,
    pub collateral_id: CollateralId,
    pub deposit_account_id: core_deposit::DepositAccountId,
    pub customer_id: CustomerId,
    pub amount: money::UsdCents,
}

/// Creates a pending facility and triggers activation by updating collateral.
/// Returns once the facility is active.
///
/// Uses the outbox event stream (`expect_event`) instead of polling, so that
/// activation is detected immediately via push notification rather than
/// sleeping in a loop — avoiding flaky timeouts under CI load.
pub async fn create_active_facility(
    ctx: &TestContext,
    terms: TermValues,
) -> anyhow::Result<ActiveFacilityState> {
    let state = create_pending_facility(ctx, terms).await?;

    let collateral_satoshis = Satoshis::from(50_000_000); // 0.5 BTC
    let effective = chrono::Utc::now().date_naive();
    let facility_id: CreditFacilityId = state.pending_facility_id.into();

    let collaterals = ctx.credit.collaterals().clone();
    let collateral_id = state.collateral_id;

    event::expect_event(
        &ctx.outbox,
        move || {
            let collaterals = collaterals.clone();
            async move {
                collaterals
                    .update_collateral_by_id(
                        &DummySubject,
                        collateral_id,
                        collateral_satoshis,
                        effective,
                    )
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityActivated { entity } if entity.id == facility_id => Some(()),
            _ => None,
        },
    )
    .await?;

    Ok(ActiveFacilityState {
        facility_id,
        collateral_id: state.collateral_id,
        deposit_account_id: state.deposit_account_id,
        customer_id: state.customer_id,
        amount: state.amount,
    })
}
