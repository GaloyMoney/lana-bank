mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::account::NewAccount;
use cala_ledger::account_set::NewAccountSet;
use cala_ledger::{AccountId, AccountSetId, CalaLedger, CalaLedgerConfig};
use core_accounting::CoreAccounting;
use helpers::{action, object};

#[tokio::test]
#[rustfmt::skip]
async fn ledger_account_ancestors() -> anyhow::Result<()> {
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
        1,,Root
        11,,Child
        11,1,Grandchild
        "#;
    let chart_id = chart.id;
    accounting.chart_of_accounts().import_from_csv(&DummySubject, chart_id, import).await?;

    let root = accounting.find_ledger_account_by_code(&DummySubject, &chart_ref, "1".to_string()).await?.unwrap();
    let child = accounting.find_ledger_account_by_code(&DummySubject, &chart_ref, "11".to_string()).await?.unwrap();
    let grandchild = accounting.find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string()).await?.unwrap();

    // chart of account
    assert_eq!(grandchild.ancestor_ids, vec![child.id, root.id]);

    let internal_id = AccountSetId::new();
    cala.account_sets()
        .create(NewAccountSet::builder().id(internal_id).name("Internal").journal_id(journal_id).build().unwrap())
        .await?;
    cala.account_sets().add_member(grandchild.id.into(), internal_id).await?;

    let leaf_id = AccountId::new();
    cala.accounts()
        .create(NewAccount::builder().id(leaf_id).code(leaf_id.to_string()).name("Leaf").build().unwrap())
        .await?;
    cala.account_sets().add_member(internal_id, leaf_id).await?;

    // internal account
    let ledger_account = accounting.find_ledger_account_by_id(&DummySubject, &chart_ref, internal_id).await?.unwrap();
    assert_eq!(ledger_account.ancestor_ids, vec![grandchild.id, child.id, root.id]);

    // leaf account with internal
    let ledger_account = accounting.find_ledger_account_by_id(&DummySubject, &chart_ref, leaf_id).await?.unwrap();
    assert_eq!(ledger_account.ancestor_ids, vec![grandchild.id, child.id, root.id]);

    let leaf2_id = AccountId::new();
    cala.accounts()
        .create(NewAccount::builder().id(leaf2_id).code(leaf2_id.to_string()).name("Leaf without internal").build().unwrap())
        .await?;
    cala.account_sets().add_member(grandchild.id.into(), leaf2_id).await?;

    // leaf account without internal
    let ledger_account = accounting.find_ledger_account_by_id(&DummySubject, &chart_ref, leaf2_id).await?.unwrap();
    assert_eq!(ledger_account.ancestor_ids, vec![grandchild.id, child.id, root.id]);

    Ok(())
}
