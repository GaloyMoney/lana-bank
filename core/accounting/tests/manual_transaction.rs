mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig, Currency, DebitOrCredit};
use core_accounting::{CoreAccounting, ManualEntryInput, manual_transactions::AccountIdOrCode};
use helpers::{action, object};
use rust_decimal_macros::dec;

#[tokio::test]
#[rustfmt::skip]
async fn manual_transaction_with_two_entries() -> anyhow::Result<()> {
    use rand::Rng;
    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder().pool(pool.clone()).exec_migrations(false).build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;

    let accounting = CoreAccounting::new(&pool, &authz, &cala, journal_id);
    let chart_ref = format!("ref-{:08}", rand::thread_rng().gen_range(0..10000));
    let chart = accounting.chart_of_accounts().create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone()).await?;
    let import = r#"
        1,,Assets
        2,,Liabilities
        "#;
    let chart_id = chart.id;
    let _ = accounting.chart_of_accounts().import_from_csv(&DummySubject, chart_id, import).await?;

    let assets = accounting.find_ledger_account_by_code(&DummySubject, &chart_ref, "1".to_string()).await?.unwrap();
    let liabilities = accounting.find_ledger_account_by_code(&DummySubject, &chart_ref, "2".to_string()).await?.unwrap();

    // account sets in chart of accounts have no members
    let assets_members = cala.account_sets().list_members_by_created_at(assets.id.into(), Default::default()).await?.entities;
    let liabilities_members = cala.account_sets().list_members_by_created_at(liabilities.id.into(), Default::default()).await?.entities;
    assert!(assets_members.is_empty());
    assert!(liabilities_members.is_empty());

    let to: AccountIdOrCode = "1".parse().unwrap();
    let from: AccountIdOrCode = "2".parse().unwrap();

    let entries = vec![
        ManualEntryInput::builder().account_id_or_code(to.clone()).amount(dec!(100)).currency(Currency::USD).direction(DebitOrCredit::Debit).description("test 1 debit").build().unwrap(),
        ManualEntryInput::builder().account_id_or_code(from.clone()).amount(dec!(100)).currency(Currency::USD).direction(DebitOrCredit::Credit).description("test 1 credit").build().unwrap(),
    ];
    accounting.execute_manual_transaction(&DummySubject, &chart_ref, None, "Test transaction 1".to_string(), entries).await?;

    // each account set in chart of accounts has one new member
    let assets_members_1 = cala.account_sets().list_members_by_created_at(assets.id.into(), Default::default()).await?.entities;
    let liabilities_members_1 = cala.account_sets().list_members_by_created_at(liabilities.id.into(), Default::default()).await?.entities;
    assert_eq!(assets_members_1.len(), 1);
    assert_eq!(liabilities_members_1.len(), 1);

    let entries = vec![
        ManualEntryInput::builder().account_id_or_code(to).amount(dec!(200)).currency(Currency::USD).direction(DebitOrCredit::Debit).description("test 2 debit").build().unwrap(),
        ManualEntryInput::builder().account_id_or_code(from).amount(dec!(200)).currency(Currency::USD).direction(DebitOrCredit::Credit).description("test 2 credit").build().unwrap(),
    ];
    accounting.execute_manual_transaction(&DummySubject, &chart_ref, None, "Test transaction 2".to_string(), entries).await?;

    // members of account set in chart of accounts did not change
    let assets_members_2 = cala.account_sets().list_members_by_created_at(assets.id.into(), Default::default()).await?.entities;
    let liabilities_members_2 = cala.account_sets().list_members_by_created_at(liabilities.id.into(), Default::default()).await?.entities;
    assert_eq!(assets_members_2.len(), 1);
    assert_eq!(liabilities_members_2.len(), 1);
    assert_eq!(assets_members_1[0].id, assets_members_2[0].id);
    assert_eq!(liabilities_members_1[0].id, liabilities_members_2[0].id);

    Ok(())
}
