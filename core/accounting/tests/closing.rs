mod helpers;

use anyhow::Result;
use chrono::{Datelike, Days, NaiveDate};
use rust_decimal::Decimal;

use authz::dummy::{DummyPerms, DummySubject};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use job::{JobSvcConfig, Jobs};

use cala_ledger::{
    AccountId, CalaLedger, CalaLedgerConfig, Currency,
    DebitOrCredit::{self, Credit, Debit},
    account::NewAccount,
};
use core_accounting::{
    AccountIdOrCode, Chart, CoreAccounting, LedgerAccountId, ManualEntryInput,
    accounting_period::{
        ChartOfAccountsIntegrationConfig,
        chart_of_accounts_integration::{AccountingPeriodConfig, Basis},
    },
};

use helpers::{action, object};

const EARNINGS: &str = "32.01";
const LOSSES: &str = "32.02";
const REVENUES: &str = "4";
const COSTS: &str = "5";
const EXPENSES: &str = "6";

#[tokio::test]
async fn annual_closing_loss() -> Result<()> {
    let mut test = setup_test().await?;

    // Revenues
    test.add_account("41.01.0101", 300, Credit).await;
    test.add_account("41.01.0102", 2400, Credit).await;

    // Costs
    test.add_account("51.01.0101", 500, Debit).await;
    test.add_account("51.01.0101", 3000, Debit).await;

    // Expenses
    test.add_account("61.01.0101", 900, Debit).await;

    assert_eq!(test.balance(REVENUES).await?, Decimal::from(300 + 2400));
    assert_eq!(test.balance(COSTS).await?, Decimal::from(500 + 3000));
    assert_eq!(test.balance(EXPENSES).await?, Decimal::from(900));

    assert!(test.children(EARNINGS).await?.is_empty());
    assert_eq!(test.balance(EARNINGS).await?, Decimal::ZERO);

    assert!(test.children(LOSSES).await?.is_empty());
    assert_eq!(test.balance(LOSSES).await?, Decimal::ZERO);

    test.accounting
        .accounting_periods()
        .close_year(&DummySubject, test.chart.id, None)
        .await?;

    assert!(test.children(EARNINGS).await?.is_empty());
    assert_eq!(test.balance(EARNINGS).await?, Decimal::ZERO);

    assert_eq!(test.children(LOSSES).await?.len(), 1);
    assert_eq!(
        test.balance(LOSSES).await?,
        Decimal::from(300 + 2400 - 500 - 3000 - 900)
    );

    assert_eq!(test.balance(REVENUES).await?, Decimal::ZERO);
    assert_eq!(test.balance(COSTS).await?, Decimal::ZERO);
    assert_eq!(test.balance(EXPENSES).await?, Decimal::ZERO);

    Ok(())
}

#[tokio::test]
async fn annual_closing_gain() -> Result<()> {
    let mut test = setup_test().await?;

    // Revenues
    test.add_account("41.01.0101", 4392, Credit).await;
    test.add_account("41.01.0102", 2058, Credit).await;

    // Costs
    test.add_account("51.01.0101", 295, Debit).await;
    test.add_account("51.01.0101", 1195, Debit).await;

    // Expenses
    test.add_account("61.01.0101", 700, Debit).await;

    assert_eq!(test.balance(REVENUES).await?, Decimal::from(4392 + 2058));
    assert_eq!(test.balance(COSTS).await?, Decimal::from(295 + 1195));
    assert_eq!(test.balance(EXPENSES).await?, Decimal::from(700));

    assert!(test.children(EARNINGS).await?.is_empty());
    assert_eq!(test.balance(EARNINGS).await?, Decimal::ZERO);

    assert!(test.children(LOSSES).await?.is_empty());
    assert_eq!(test.balance(LOSSES).await?, Decimal::ZERO);

    test.accounting
        .accounting_periods()
        .close_year(&DummySubject, test.chart.id, None)
        .await?;

    assert_eq!(test.children(EARNINGS).await?.len(), 1);
    assert_eq!(
        test.balance(EARNINGS).await?,
        Decimal::from(4392 + 2058 - 295 - 1195 - 700)
    );

    assert!(test.children(LOSSES).await?.is_empty());
    assert_eq!(test.balance(LOSSES).await?, Decimal::ZERO);

    assert_eq!(test.balance(REVENUES).await?, Decimal::ZERO);
    assert_eq!(test.balance(COSTS).await?, Decimal::ZERO);
    assert_eq!(test.balance(EXPENSES).await?, Decimal::ZERO);

    Ok(())
}

async fn setup_test() -> anyhow::Result<Test> {
    use rand::Rng;
    let pool = helpers::init_pool().await?;

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);
    let jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(&pool, &authz, &cala, journal_id, document_storage, &jobs);
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let chart = accounting
        .chart_of_accounts()
        .create_chart(
            &DummySubject,
            "Test chart".to_string(),
            chart_ref.clone(),
            "2021-01-01".parse::<chrono::NaiveDate>().unwrap(),
        )
        .await?;
    let import = r#"
1,,,Assets,Debit,
,,,,,
11,,,Current Assets,,
,,,,,
,01,,Cash and Equivalents,,
,,,,,
,,0101,Operating Cash,,
,,,,,
,,0102,Petty Cash,,
,,,,,
,02,,Receivables,,
,,,,,
,,0201,Interest Receivable,,
,,,,,
,,0202,Loans Receivable,,
,,,,,
,,0203,Overdue Loans Receivable,,
,,,,,
,03,,Inventory,,
,,,,,
,,0301,Raw Materials,,
,,,,,
,,0302,Work In Progress,,
,,,,,
,,0303,Finished Goods,,
,,,,,
12,,,Non-Current Assets,,
,,,,,
,01,,Property and Equipment,,
,,,,,
,,0101,Land,,
,,,,,
,,0102,Buildings,,
,,,,,
,,0103,Equipment,,
,,,,,
,02,,Intangible Assets,,
,,,,,
,,0201,Goodwill,,
,,,,,
,,0202,Intellectual Property,,
,,,,,
3,,,Equity,Credit,
,,,,,
31,,,Contributed Capital,,
,,,,,
,01,,Common Stock,,
,,,,,
,02,,Preferred Stock,,
,,,,,
32,,,Retained Earnings,,
,,,,,
,01,,Prior Year Earnings,,
,,,,,
,02,,Prior Year Losses,,
,,,,,
4,,,Revenue,Credit,
,,,,,
41,,,Operating Revenue,,
,,,,,
,01,,Sales Revenue,,
,,,,,
,,0101,Product A Sales,,
,,,,,
,,0102,Product B Sales,,
,,,,,
,02,,Service Revenue,,
,,,,,
,,0201,Consulting Services,,
,,,,,
,,0202,Maintenance Services,,
,,,,,
5,,,Cost of Revenue,Debit,
,,,,,
51,,,Capital Cost,,
,,,,,
,01,,Custody,,
,,,,,
,,0101,Custodian Fees,,
,,,,,
6,,,Expenses,Debit,
,,,,,
61,,,Fixed Expenses,,
,,,,,
,01,,Regulatory,,
,,,,,
,,0101,Regulatory Fees,,
        "#;
    let chart_id = chart.id;
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    accounting
        .accounting_periods()
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            &chart,
            ChartOfAccountsIntegrationConfig {
                chart_of_accounts_id: chart_id,
                revenue_code: "4".parse().unwrap(),
                cost_of_revenue_code: "5".parse().unwrap(),
                expenses_code: "6".parse().unwrap(),
                equity_retained_earnings_code: "32.01".parse().unwrap(),
                equity_retained_losses_code: "32.02".parse().unwrap(),
                accounting_periods: vec![],
            },
        )
        .await?;

    // Calculate period start and end so that we are now in the middle of grace period.

    let today = es_entity::prelude::sim_time::now().date_naive();
    let period_end = today.checked_sub_days(Days::new(2)).unwrap();
    let central_date = period_end.checked_sub_days(Days::new(1)).unwrap();

    accounting
        .accounting_periods()
        .open_initial_periods(
            chart.id,
            chart.account_set_id,
            central_date,
            vec![
                AccountingPeriodConfig {
                    basis: Basis::Monthly {
                        day: period_end.day(),
                    },
                    grace_period_days: 5,
                },
                AccountingPeriodConfig {
                    basis: Basis::Annual {
                        day: period_end.day(),
                        month: period_end.month(),
                    },
                    grace_period_days: 10,
                },
            ],
        )
        .await?;

    Ok(Test {
        accounting,
        chart,
        cala,
        inner_date: central_date,
        accounts: vec![],
    })
}

struct Test {
    pub cala: CalaLedger,
    pub accounting: CoreAccounting<DummyPerms<action::DummyAction, object::DummyObject>>,
    pub chart: Chart,
    pub accounts: Vec<AccountId>,
    pub inner_date: NaiveDate,
}

impl Test {
    pub async fn add_account(&mut self, parent: &str, funds: i32, balance_type: DebitOrCredit) {
        let account_id = AccountId::new();
        let _ = self
            .cala
            .accounts()
            .create(
                NewAccount::builder()
                    .id(account_id)
                    .code(account_id.to_string())
                    .name(format!("Account {}", self.accounts.len()))
                    .normal_balance_type(balance_type)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = self
            .cala
            .account_sets()
            .add_member(
                self.chart
                    .account_set_id_from_code(&parent.parse().unwrap())
                    .unwrap(),
                account_id,
            )
            .await
            .unwrap();

        // Fund the account

        let (source_dir, dest_dir) = match balance_type {
            DebitOrCredit::Debit if funds >= 0 => (DebitOrCredit::Credit, DebitOrCredit::Debit),
            DebitOrCredit::Debit => (DebitOrCredit::Debit, DebitOrCredit::Credit),
            DebitOrCredit::Credit if funds >= 0 => (DebitOrCredit::Debit, DebitOrCredit::Credit),
            DebitOrCredit::Credit => (DebitOrCredit::Credit, DebitOrCredit::Debit),
        };

        self.accounting
            .execute_manual_transaction(
                &DummySubject,
                &self.chart.reference,
                None,
                format!("Transaction {}", self.accounts.len()),
                Some(self.inner_date),
                vec![
                    ManualEntryInput::builder()
                        .account_id_or_code(AccountIdOrCode::Code("11.01.0101".parse().unwrap()))
                        .amount(funds.abs().into())
                        .currency(Currency::USD)
                        .direction(source_dir)
                        .description(format!("Source {}", self.accounts.len()))
                        .build()
                        .unwrap(),
                    ManualEntryInput::builder()
                        .account_id_or_code(AccountIdOrCode::Id(account_id.into()))
                        .amount(funds.abs().into())
                        .currency(Currency::USD)
                        .direction(dest_dir)
                        .description(format!("Destination {}", self.accounts.len()))
                        .build()
                        .unwrap(),
                ],
            )
            .await
            .unwrap();

        self.accounts.push(account_id);
    }

    pub async fn balance(&self, code: &str) -> Result<Decimal> {
        let account = self
            .accounting
            .find_ledger_account_by_code(&DummySubject, &self.chart.reference, code.to_string())
            .await?
            .unwrap();

        Ok(account
            .usd_balance_range
            .and_then(|r| r.close)
            .map(|b| b.settled())
            .unwrap_or(Decimal::ZERO))
    }

    pub async fn children(&self, code: &str) -> Result<Vec<LedgerAccountId>> {
        let account = self
            .accounting
            .find_ledger_account_by_code(&DummySubject, &self.chart.reference, code.to_string())
            .await?
            .unwrap();

        Ok(account.children_ids)
    }
}
