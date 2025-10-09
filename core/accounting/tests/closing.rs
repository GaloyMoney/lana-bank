mod helpers;

use std::collections::HashMap;

use authz::dummy::{DummyPerms, DummySubject};
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use job::{Jobs, JobsConfig};

use cala_ledger::{
    AccountId, AccountSetId, CalaLedger, CalaLedgerConfig, Currency, DebitOrCredit, JournalId,
    account::NewAccount,
    account_set::{AccountSetMemberId, error::AccountSetError},
};
use core_accounting::{AccountIdOrCode, Chart, CoreAccounting, ManualEntryInput};
use helpers::{action, object};
use rust_decimal::Decimal;

// TODO: Re-evaluate after merge with config work.
struct PlaceholderAnnualClosingTxConfig {
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
    equity_retained_earnings_code: String,
    equity_retained_losses_code: String,
}

impl PlaceholderAnnualClosingTxConfig {
    pub fn new() -> Self {
        Self {
            revenue_code: "4".to_string(),
            cost_of_revenue_code: "5".to_string(),
            expenses_code: "6".to_string(),
            equity_retained_earnings_code: "3140.01.0101".to_string(),
            equity_retained_losses_code: "3140.01.0201".to_string(),
        }
    }
}

#[tokio::test]
async fn annual_closing() -> anyhow::Result<()> {
    let mut test = prepare_test().await?;
    // TODO: Add more complex setup (Cost of Revenues, Expenses).
    test.account("11.01.0101", 100).await;
    test.account("11.03.0302", 200).await;
    test.account("31.01", 300).await;
    test.account("41.01.0102", 400).await;

    let year_end_balances = test.balances().await;
    // TODO: Move to prepare or make a helper when multiple cases on annual closing transaction.
    let _closed_chart = test.accounting
        .chart_of_accounts()
        .close_monthly(&DummySubject, test.chart.id)
        .await?;

    let _ann_closing_tx = test
        .accounting
        .annual_closing_transactions()
        .execute(
            &DummySubject, 
            test.chart.id,
            None,
            "Test Annual Closing".to_string(),
        )
        .await?;

    let year_end_balances_after = test.balances().await;
    println!("{:#?}", year_end_balances_after);

    // TODO: Make assertions based on account ID.

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
    let jobs = Jobs::new(&pool, JobsConfig::default());

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
,01,,Current Year Earnings,,
,,,,,
,02,,Prior Years Earnings,,
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

    let source = AccountId::new();
    let _ = cala
        .accounts()
        .create(
            NewAccount::builder()
                .id(source)
                .code(source.to_string())
                .name("Source")
                .build()
                .unwrap(),
        )
        .await?;

    Ok(Test {
        accounting,
        journal_id,
        chart,
        cala,
        source,
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

struct Test {
    pub cala: CalaLedger,
    pub accounting: CoreAccounting<DummyPerms<action::DummyAction, object::DummyObject>>,
    pub chart: Chart,
    pub source: AccountId,
    pub journal_id: JournalId,
    pub accounts: Vec<AccountId>,
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

    pub async fn account(&mut self, parent: &str, funds: u32) {
        let account_id = AccountId::new();
        let _ = self
            .cala
            .accounts()
            .create(
                NewAccount::builder()
                    .id(account_id)
                    .code(account_id.to_string())
                    .name(format!("Account {}", self.accounts.len()))
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

        let entries = vec![
            ManualEntryInput::builder()
                .account_id_or_code(AccountIdOrCode::Id(self.source.into()))
                .amount(funds.into())
                .currency(Currency::USD)
                .direction(DebitOrCredit::Debit)
                .description(format!("Debit {}", self.accounts.len()))
                .build()
                .unwrap(),
            ManualEntryInput::builder()
                .account_id_or_code(AccountIdOrCode::Id(account_id.into()))
                .amount(funds.into())
                .currency(Currency::USD)
                .direction(DebitOrCredit::Credit)
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
                None,
                entries,
            )
            .await
            .unwrap();

        self.accounts.push(account_id);
    }
}
