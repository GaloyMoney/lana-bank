mod helpers;

use anyhow::Result;
use chrono::{Days, NaiveDate};
use rust_decimal::Decimal;

use authz::dummy::{DummyPerms, DummySubject};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use domain_config::DomainConfigs;
use job::{JobSvcConfig, Jobs};

use cala_ledger::{
    AccountId, CalaLedger, CalaLedgerConfig, Currency,
    DebitOrCredit::{self, Credit, Debit},
    account::NewAccount,
};
use core_accounting::{
    AccountCode, AccountIdOrCode, CalaTxId, Chart, ClosingAccountCodes, ClosingTxDetails, fiscal_year::FiscalYearRepo,
    CoreAccounting, LedgerAccountId, ManualEntryInput, ProfitAndLossStatement,
    balance_sheet::ChartOfAccountsIntegrationConfig as BalanceSheetConfig, fiscal_year::{FiscalYear, FiscalYearRepo},
    profit_and_loss::ChartOfAccountsIntegrationConfig as ProfitAndLossConfig,
};

use helpers::{action, object};

const ASSETS: &str = "1";
const LIABILITIES: &str = "2";
const EQUITY: &str = "3";
const RETAINED_EARNINGS_GAIN: &str = "32.01";
const RETAINED_EARNINGS_LOSS: &str = "32.02";
const REVENUES: &str = "4";
const COSTS: &str = "5";
const EXPENSES: &str = "6";

#[tokio::test]
async fn post_closing_tx_with_gain() -> Result<()> {
    const EXPECTED_CREDIT_NORMAL_NET_INCOME: i32 = 100;
    let mut test = setup_test().await?;

    // Revenues
    test.add_account_with_balance("41.01.0101", 400, Credit)
        .await;
    test.add_account_with_balance("41.01.0102", 100, Credit)
        .await;

    // Costs
    test.add_account_with_balance("51.01.0101", 200, Debit)
        .await;

    // Expenses
    test.add_account_with_balance("61.01.0101", 200, Debit)
        .await;

    assert_eq!(test.balance(REVENUES).await?, Decimal::from(400 + 100));
    assert_eq!(test.balance(COSTS).await?, Decimal::from(200));
    assert_eq!(test.balance(EXPENSES).await?, Decimal::from(200));

    assert!(test.children(RETAINED_EARNINGS_GAIN).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_GAIN).await?, Decimal::ZERO);

    assert!(test.children(RETAINED_EARNINGS_LOSS).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_LOSS).await?, Decimal::ZERO);

    let ledger_tx_id = CalaTxId::new();
    let effective_balances_from = test.fiscal_year.opened_as_of;
    let effective_balances_as_of = test.fiscal_year.closes_as_of();
    let closing_account_codes = ClosingAccountCodes {
        revenue: REVENUES.parse::<AccountCode>().unwrap(),
        cost_of_revenue: COSTS.parse::<AccountCode>().unwrap(),
        expenses: EXPENSES.parse::<AccountCode>().unwrap(),
        equity_retained_earnings: RETAINED_EARNINGS_GAIN.parse::<AccountCode>().unwrap(),
        equity_retained_losses: RETAINED_EARNINGS_LOSS.parse::<AccountCode>().unwrap(),
    };
    let closing_tx_details = ClosingTxDetails {
        description: test.fiscal_year.reference.clone(),
        tx_id: ledger_tx_id,
        effective_balances_until: effective_balances_as_of,
        effective_balances_from,
    };

    let op = test.fiscal_year_repo.begin_op().await.unwrap();
    test.accounting
        .chart_of_accounts()
        .post_closing_transaction(op, test.chart.id, closing_account_codes, closing_tx_details)
        .await?;

    assert!(test.children(RETAINED_EARNINGS_LOSS).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_LOSS).await?, Decimal::ZERO);

    let net_income_account_ids = test.children(RETAINED_EARNINGS_GAIN).await?;
    assert_eq!(net_income_account_ids.len(), 1);

    let net_income_gain = Decimal::from(EXPECTED_CREDIT_NORMAL_NET_INCOME);
    assert_eq!(
        test.balance_by_account_id(net_income_account_ids[0])
            .await?,
        net_income_gain
    );

    assert_eq!(test.balance(REVENUES).await?, Decimal::ZERO);
    assert_eq!(test.balance(COSTS).await?, Decimal::ZERO);
    assert_eq!(test.balance(EXPENSES).await?, Decimal::ZERO);

    let until = test.fiscal_year.closes_as_of();

    let profit_and_loss_normal_balance_before_close_tx = test
        .pl_statement(until, Some(until))
        .await?
        .usd_balance_range
        .as_ref()
        .expect("USD balance range is missing for P&L statement")
        .open
        .clone()
        .unwrap();

    let profit_and_loss_normal_balance_after_close_tx = test
        .pl_statement(until, Some(until))
        .await?
        .usd_balance_range
        .as_ref()
        .expect("USD balance range is missing for P&L statement")
        .close
        .clone()
        .unwrap();

    assert_eq!(
        profit_and_loss_normal_balance_before_close_tx.settled(),
        Decimal::from(EXPECTED_CREDIT_NORMAL_NET_INCOME)
    );
    assert_eq!(
        profit_and_loss_normal_balance_after_close_tx.settled(),
        Decimal::ZERO
    );

    Ok(())
}

#[tokio::test]
async fn post_closing_tx_with_loss() -> Result<()> {
    const EXPECTED_DEBIT_NORMAL_NET_INCOME: i32 = 100;
    let expected_credit_normal_net_income: i32 = -100;

    let mut test = setup_test().await?;

    // Revenues
    test.add_account_with_balance("41.01.0101", 300, Credit)
        .await;
    test.add_account_with_balance("41.01.0102", 200, Credit)
        .await;

    // Costs
    test.add_account_with_balance("51.01.0101", 250, Debit)
        .await;
    test.add_account_with_balance("51.01.0101", 250, Debit)
        .await;

    // Expenses
    test.add_account_with_balance("61.01.0101", 100, Debit)
        .await;

    assert_eq!(test.balance(REVENUES).await?, Decimal::from(300 + 200));
    assert_eq!(test.balance(COSTS).await?, Decimal::from(250 + 250));
    assert_eq!(test.balance(EXPENSES).await?, Decimal::from(100));

    assert!(test.children(RETAINED_EARNINGS_GAIN).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_GAIN).await?, Decimal::ZERO);

    assert!(test.children(RETAINED_EARNINGS_LOSS).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_LOSS).await?, Decimal::ZERO);

    let ledger_tx_id = CalaTxId::new();
    let effective_balances_from = test.fiscal_year.opened_as_of;
    let effective_balances_as_of = test.fiscal_year.closes_as_of();
    let closing_account_codes = ClosingAccountCodes {
        revenue: REVENUES.parse::<AccountCode>().unwrap(),
        cost_of_revenue: COSTS.parse::<AccountCode>().unwrap(),
        expenses: EXPENSES.parse::<AccountCode>().unwrap(),
        equity_retained_earnings: RETAINED_EARNINGS_GAIN.parse::<AccountCode>().unwrap(),
        equity_retained_losses: RETAINED_EARNINGS_LOSS.parse::<AccountCode>().unwrap(),
    };
    let closing_spec = ClosingTxDetails {
        description: test.fiscal_year.reference.clone(),
        tx_id: ledger_tx_id,
        effective_balances_until: effective_balances_as_of,
        effective_balances_from,
    };

    let op = test.fiscal_year_repo.begin_op().await.unwrap();
    test.accounting
        .chart_of_accounts()
        .post_closing_transaction(op, test.chart.id, closing_account_codes, closing_spec)
        .await?;
    assert!(test.children(RETAINED_EARNINGS_GAIN).await?.is_empty());
    assert_eq!(test.balance(RETAINED_EARNINGS_GAIN).await?, Decimal::ZERO);

    let net_income_account_ids = test.children(RETAINED_EARNINGS_LOSS).await?;
    assert_eq!(net_income_account_ids.len(), 1);

    let net_income_loss = Decimal::from(EXPECTED_DEBIT_NORMAL_NET_INCOME);
    assert_eq!(
        test.balance_by_account_id(net_income_account_ids[0])
            .await?,
        net_income_loss
    );

    assert_eq!(test.balance(REVENUES).await?, Decimal::ZERO);
    assert_eq!(test.balance(COSTS).await?, Decimal::ZERO);
    assert_eq!(test.balance(EXPENSES).await?, Decimal::ZERO);

    let until = test.fiscal_year.closes_as_of();
    let profit_and_loss_normal_balance_before_close_tx = test
        .pl_statement(until, Some(until))
        .await?
        .usd_balance_range
        .as_ref()
        .expect("USD balance range is missing for P&L statement")
        .open
        .clone()
        .unwrap();

    let profit_and_loss_normal_balance_after_close_tx = test
        .pl_statement(until, Some(until))
        .await?
        .usd_balance_range
        .as_ref()
        .expect("USD balance range is missing for P&L statement")
        .close
        .clone()
        .unwrap();

    assert_eq!(
        profit_and_loss_normal_balance_before_close_tx.settled(),
        Decimal::from(expected_credit_normal_net_income)
    );
    assert_eq!(
        profit_and_loss_normal_balance_after_close_tx.settled(),
        Decimal::ZERO
    );
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
    let domain_configs = DomainConfigs::new(&pool);
    let journal_id = helpers::init_journal(&cala).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage);
    let jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let fiscal_year_repo = FiscalYearRepo::new(&pool);
    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &jobs,
        &domain_configs,
    );
    let chart_ref = format!("ref-{:08}", rand::rng().random_range(0..10000));
    let balance_sheet_name = format!("Test Balance Sheet #{}", rand::rng().random_range(0..10000));
    let pl_statement_name = format!(
        "Test Profit & Loss Statement #{}",
        rand::rng().random_range(0..10000)
    );
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
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
2,,,Liabilities,Debit,
,,,,,
21,,,Current Liabilities,,
,,,,,
,01,,Accounts Payable,,
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
,01,,Annual Gains,,
,,,,,
,02,,Annual Losses,,
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
    let (chart, _) = accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, &chart.reference, import)
        .await?;

    let opened_as_of: NaiveDate = "2021-12-01".parse::<NaiveDate>().unwrap();
    let fiscal_year = accounting
        .init_fiscal_year_for_chart(&DummySubject, &chart_ref, opened_as_of)
        .await?;

    accounting
        .balance_sheets()
        .create_balance_sheet(balance_sheet_name.clone())
        .await?;
    let balance_sheet_config = BalanceSheetConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_assets_code: ASSETS.parse().unwrap(),
        chart_of_accounts_liabilities_code: LIABILITIES.parse().unwrap(),
        chart_of_accounts_equity_code: EQUITY.parse().unwrap(),
        chart_of_accounts_revenue_code: REVENUES.parse().unwrap(),
        chart_of_accounts_cost_of_revenue_code: COSTS.parse().unwrap(),
        chart_of_accounts_expenses_code: EXPENSES.parse().unwrap(),
    };
    accounting
        .balance_sheets()
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            balance_sheet_name,
            &chart,
            balance_sheet_config,
        )
        .await?;

    accounting
        .profit_and_loss()
        .create_pl_statement(pl_statement_name.clone())
        .await?;
    let pl_statement_config = ProfitAndLossConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_revenue_code: REVENUES.parse().unwrap(),
        chart_of_accounts_cost_of_revenue_code: COSTS.parse().unwrap(),
        chart_of_accounts_expenses_code: EXPENSES.parse().unwrap(),
    };
    accounting
        .profit_and_loss()
        .set_chart_of_accounts_integration_config(
            &DummySubject,
            pl_statement_name.clone(),
            &chart,
            pl_statement_config,
        )
        .await?;

    Ok(Test {
        accounting,
        chart,
        cala,
        fiscal_year,
        fiscal_year_repo,
        accounts: vec![],
        pl_statement_name,
    })
}

struct Test {
    pub cala: CalaLedger,
    pub accounting: CoreAccounting<DummyPerms<action::DummyAction, object::DummyObject>>,
    pub chart: Chart,
    pub fiscal_year: FiscalYear,
    pub fiscal_year_repo: FiscalYearRepo,
    pub accounts: Vec<AccountId>,
    pub pl_statement_name: String,
}

impl Test {
    pub async fn add_account_with_balance(
        &mut self,
        parent: &str,
        funds: i32,
        balance_type: DebitOrCredit,
    ) {
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

        let (source_dir, dest_dir) = match balance_type {
            DebitOrCredit::Debit if funds >= 0 => (DebitOrCredit::Credit, DebitOrCredit::Debit),
            DebitOrCredit::Debit => (DebitOrCredit::Debit, DebitOrCredit::Credit),
            DebitOrCredit::Credit if funds >= 0 => (DebitOrCredit::Debit, DebitOrCredit::Credit),
            DebitOrCredit::Credit => (DebitOrCredit::Credit, DebitOrCredit::Debit),
        };
        let effective_tx_date = self
            .fiscal_year
            .opened_as_of
            .checked_add_days(Days::new(15))
            .unwrap();
        self.accounting
            .execute_manual_transaction(
                &DummySubject,
                &self.chart.reference,
                None,
                format!("Transaction {}", self.accounts.len()),
                Some(effective_tx_date),
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

    pub async fn balance_by_account_id(&self, account_id: LedgerAccountId) -> Result<Decimal> {
        let account = self
            .accounting
            .find_ledger_account_by_id(&DummySubject, &self.chart.reference, account_id)
            .await?
            .unwrap();
        Ok(account
            .usd_balance_range
            .and_then(|r| r.close)
            .map(|b| b.settled())
            .unwrap_or(Decimal::ZERO))
    }

    pub async fn pl_statement(
        &self,
        from: NaiveDate,
        until: Option<NaiveDate>,
    ) -> Result<ProfitAndLossStatement> {
        Ok(self
            .accounting
            .profit_and_loss()
            .pl_statement(&DummySubject, self.pl_statement_name.clone(), from, until)
            .await?)
    }
}
