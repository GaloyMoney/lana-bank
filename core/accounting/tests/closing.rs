mod helpers;

use std::collections::HashMap;

use authz::dummy::{DummyPerms, DummySubject};
use chrono::{Datelike, Days, NaiveDate};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use job::{JobSvcConfig, Jobs};

use cala_ledger::{
    AccountId, AccountSetId, CalaLedger, CalaLedgerConfig, Currency, DebitOrCredit, JournalId,
    account::NewAccount,
    account_set::{AccountSetMemberId, error::AccountSetError},
    balance::error::BalanceError,
};
use core_accounting::{
    AccountCode, AccountIdOrCode, Chart, CoreAccounting, ManualEntryInput,
    accounting_period::{
        ChartOfAccountsIntegrationConfig,
        chart_of_accounts_integration::{AccountingPeriodConfig, Basis},
    },
};
use helpers::{action, object};
use rust_decimal::Decimal;

#[tokio::test]
async fn closing() -> anyhow::Result<()> {
    let mut test = prepare_test().await?;

    test.account("41.01.0102", 300, DebitOrCredit::Credit).await;
    test.account("51.01.0101", 500, DebitOrCredit::Debit).await;
    test.account("61.01.0101", 900, DebitOrCredit::Debit).await;

    let pre_close_balances = test.balances().await;

    let equity_parent_account_set_id = test
        .chart
        .account_set_id_from_code(&"32".parse::<AccountCode>().unwrap())?;

    let pre_equity_accounts =
        find_all_accounts(&test.cala, equity_parent_account_set_id.clone()).await?;

    // let _closed_chart = test
    //     .accounting
    //     .accounting_periods()
    //     .close_month(&DummySubject, test.chart.id)
    //     .await?;

    let _ann_closing_tx = test
        .accounting
        .accounting_periods()
        .close_year(
            &DummySubject,
            test.chart.id,
            Some("Test Annual Closing".to_string()),
        )
        .await?;

    let post_close_balances = test.balances().await;

    for (act, _pre_bal) in &pre_close_balances {
        if let Some(post_bal) = post_close_balances.get(act) {
            assert_eq!(*post_bal, Decimal::ZERO);
        }
    }

    let post_equity_accounts =
        find_all_accounts(&test.cala, equity_parent_account_set_id.clone()).await?;
    assert_eq!(post_equity_accounts.len(), pre_equity_accounts.len() + 1);

    let post_equity_balance =
        find_account_balance(&test.cala, post_equity_accounts[0], test.journal_id).await?;
    assert_eq!(post_equity_balance, Decimal::from(1100));

    Ok(())
}

async fn prepare_test() -> anyhow::Result<Test> {
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
            chart_id,
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
        journal_id,
        chart,
        cala,
        inner_date: central_date,
        accounts: vec![],
    })
}

pub async fn find_all_accounts(
    cala: &CalaLedger,
    id: AccountSetId,
) -> Result<Vec<AccountId>, AccountSetError> {
    let children = cala
        .account_sets()
        .list_members_by_external_id(id, Default::default())
        .await?
        .entities;

    let mut results = Vec::new();

    for child in children {
        match child.id {
            AccountSetMemberId::Account(id) => {
                results.push(id);
            }
            AccountSetMemberId::AccountSet(id) => {
                let nested_children = Box::pin(find_all_accounts(cala, id)).await?;
                results.extend(nested_children);
            }
        }
    }

    Ok(results)
}

pub async fn find_account_balance(
    cala: &CalaLedger,
    id: impl Into<AccountId>,
    journal_id: JournalId,
) -> Result<Decimal, BalanceError> {
    let balance = cala
        .balances()
        .find(journal_id, id.into(), Currency::USD)
        .await?;

    Ok(balance.settled())
}

struct Test {
    pub cala: CalaLedger,
    pub accounting: CoreAccounting<DummyPerms<action::DummyAction, object::DummyObject>>,
    pub chart: Chart,
    pub journal_id: JournalId,
    pub accounts: Vec<AccountId>,
    pub inner_date: NaiveDate,
}

impl Test {
    pub async fn balances(&self) -> HashMap<AccountId, Decimal> {
        let ids = self
            .accounts
            .iter()
            .copied()
            .map(|id| (self.journal_id, id, Currency::USD))
            .collect::<Vec<_>>();

        self.cala
            .balances()
            .find_all(&ids)
            .await
            .unwrap()
            .into_iter()
            .map(|((_, id, _), b)| (id, b.settled()))
            .collect()
    }

    pub async fn account(&mut self, parent: &str, funds: u32, balance_type: DebitOrCredit) {
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

        let (source_dir, dest_dir) = if balance_type == DebitOrCredit::Debit {
            (DebitOrCredit::Credit, DebitOrCredit::Debit)
        } else {
            (DebitOrCredit::Debit, DebitOrCredit::Credit)
        };

        let entries = vec![
            ManualEntryInput::builder()
                .account_id_or_code(AccountIdOrCode::Code("11.01.0101".parse().unwrap()))
                .amount(funds.into())
                .currency(Currency::USD)
                .direction(source_dir)
                .description(format!("Debit {}", self.accounts.len()))
                .build()
                .unwrap(),
            ManualEntryInput::builder()
                .account_id_or_code(AccountIdOrCode::Id(account_id.into()))
                .amount(funds.into())
                .currency(Currency::USD)
                .direction(dest_dir)
                .description(format!("Credit {}", self.accounts.len()))
                .build()
                .unwrap(),
        ];
        self.accounting
            .execute_manual_transaction(
                &DummySubject,
                &self.chart.reference,
                None,
                format!("Transaction {}", self.accounts.len()),
                Some(self.inner_date),
                entries,
            )
            .await
            .unwrap();

        self.accounts.push(account_id);
    }
}
