mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{
    AccountId, AccountSetId, CalaLedger, CalaLedgerConfig,
    account::NewAccount,
    account_set::{AccountSetMemberId, NewAccountSet},
};
use cloud_storage::{Storage, config::StorageConfig};
use core_accounting::CoreAccounting;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use helpers::{action, default_accounting_base_config, object};
use job::{JobSvcConfig, Jobs};

#[tokio::test]
async fn ledger_account_ancestors() -> anyhow::Result<()> {
    use uuid::Uuid;
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;
    let outbox = helpers::init_outbox(&pool).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &outbox,
    );
    let chart_ref = format!("ref-{}", Uuid::new_v4());
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let (balance_sheet_name, pl_name, tb_name) =
        helpers::create_test_statements(&accounting).await?;

    let import = format!(
        "{}{}",
        helpers::BASE_ACCOUNTS_CSV,
        r#"
    11,,,Child,,
    ,1,,Grandchild,,
    "#
    );
    let base_config = default_accounting_base_config();
    accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name,
            &pl_name,
            &tb_name,
        )
        .await?;

    let root = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "1".to_string())
        .await?
        .unwrap();
    let child = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11".to_string())
        .await?
        .unwrap();
    let grandchild = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .unwrap();

    // chart of account
    assert_eq!(grandchild.ancestor_ids, vec![child.id, root.id]);

    let internal_id = AccountSetId::new();
    cala.account_sets()
        .create(
            NewAccountSet::builder()
                .id(internal_id)
                .name("Internal")
                .journal_id(journal_id)
                .build()
                .unwrap(),
        )
        .await?;
    cala.account_sets()
        .add_member(grandchild.id.into(), internal_id)
        .await?;

    let leaf_id = AccountId::new();
    cala.accounts()
        .create(
            NewAccount::builder()
                .id(leaf_id)
                .code(leaf_id.to_string())
                .name("Leaf")
                .build()
                .unwrap(),
        )
        .await?;
    cala.account_sets().add_member(internal_id, leaf_id).await?;

    // internal account
    let ledger_account = accounting
        .find_ledger_account_by_id(&DummySubject, &chart_ref, internal_id)
        .await?
        .unwrap();
    assert_eq!(
        ledger_account.ancestor_ids,
        vec![grandchild.id, child.id, root.id]
    );

    // leaf account with internal
    let ledger_account = accounting
        .find_ledger_account_by_id(&DummySubject, &chart_ref, leaf_id)
        .await?
        .unwrap();
    assert_eq!(
        ledger_account.ancestor_ids,
        vec![grandchild.id, child.id, root.id]
    );

    let leaf2_id = AccountId::new();
    cala.accounts()
        .create(
            NewAccount::builder()
                .id(leaf2_id)
                .code(leaf2_id.to_string())
                .name("Leaf without internal")
                .build()
                .unwrap(),
        )
        .await?;
    cala.account_sets()
        .add_member(grandchild.id.into(), leaf2_id)
        .await?;

    // leaf account without internal
    let ledger_account = accounting
        .find_ledger_account_by_id(&DummySubject, &chart_ref, leaf2_id)
        .await?
        .unwrap();
    assert_eq!(
        ledger_account.ancestor_ids,
        vec![grandchild.id, child.id, root.id]
    );

    Ok(())
}

#[tokio::test]
async fn ledger_account_children() -> anyhow::Result<()> {
    use uuid::Uuid;
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;
    let outbox = helpers::init_outbox(&pool).await?;

    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &outbox,
    );
    let chart_ref = format!("ref-{}", Uuid::new_v4());
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let (balance_sheet_name, pl_name, tb_name) =
        helpers::create_test_statements(&accounting).await?;

    let import = format!(
        "{}{}",
        helpers::BASE_ACCOUNTS_CSV,
        r#"
    11,,,Child,,
    ,1,,Grandchild,,
    "#
    );
    let base_config = default_accounting_base_config();
    accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name,
            &pl_name,
            &tb_name,
        )
        .await?;
    let root = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "1".to_string())
        .await?
        .unwrap();
    let child = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11".to_string())
        .await?
        .unwrap();
    let grandchild = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .unwrap();

    // chart of account
    assert_eq!(root.children_ids, vec![child.id]);
    assert_eq!(child.children_ids, vec![grandchild.id]);
    assert_eq!(grandchild.children_ids, vec![]);

    let internal_id = AccountSetId::new();
    cala.account_sets()
        .create(
            NewAccountSet::builder()
                .id(internal_id)
                .name("Internal")
                .journal_id(journal_id)
                .build()
                .unwrap(),
        )
        .await?;
    cala.account_sets()
        .add_member(grandchild.id.into(), internal_id)
        .await?;

    let grandchild = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .unwrap();
    assert_eq!(grandchild.children_ids, vec![]);

    let leaf_id = AccountId::new();
    cala.accounts()
        .create(
            NewAccount::builder()
                .id(leaf_id)
                .code(leaf_id.to_string())
                .name("Leaf")
                .build()
                .unwrap(),
        )
        .await?;
    cala.account_sets().add_member(internal_id, leaf_id).await?;

    let grandchild = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .unwrap();
    assert_eq!(grandchild.children_ids, vec![leaf_id.into()]);

    Ok(())
}

#[tokio::test]
async fn internal_account_contains_coa_account() -> anyhow::Result<()> {
    use uuid::Uuid;
    let pool = helpers::init_pool().await?;
    let (clock, _) = ClockHandle::artificial(ArtificialClockConfig::manual());
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;
    let outbox = helpers::init_outbox(&pool).await?;
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let mut jobs = Jobs::init(JobSvcConfig::builder().pool(pool.clone()).build().unwrap()).await?;

    let accounting = CoreAccounting::new(
        &pool,
        &authz,
        &cala,
        journal_id,
        document_storage,
        &mut jobs,
        &outbox,
    );
    let chart_ref = format!("ref-{}", Uuid::new_v4());
    accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let (balance_sheet_name, pl_name, tb_name) =
        helpers::create_test_statements(&accounting).await?;

    let import = format!(
        "{}{}",
        helpers::BASE_ACCOUNTS_CSV,
        r#"
    11,,,Child,,
    ,1,,Grandchild,,
    "#
    );
    let base_config = default_accounting_base_config();
    accounting
        .import_csv_with_base_config(
            &DummySubject,
            &chart_ref,
            import,
            base_config,
            &balance_sheet_name,
            &pl_name,
            &tb_name,
        )
        .await?;
    let root = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "1".to_string())
        .await?
        .unwrap();
    let child = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11".to_string())
        .await?
        .unwrap();
    let grandchild = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .unwrap();

    // chart of account
    assert_eq!(root.children_ids, vec![child.id]);
    assert_eq!(child.children_ids, vec![grandchild.id]);
    assert_eq!(grandchild.children_ids, vec![]);

    let module_specific_account_set_id = AccountSetId::new();
    cala.account_sets()
        .create(
            NewAccountSet::builder()
                .id(module_specific_account_set_id)
                .name("Internal")
                .journal_id(journal_id)
                .build()
                .unwrap(),
        )
        .await?;

    cala.account_sets()
        .add_member(
            module_specific_account_set_id,
            AccountSetMemberId::AccountSet(child.id.into()),
        )
        .await?;

    let module_specific_account = accounting
        .find_ledger_account_by_id(&DummySubject, &chart_ref, module_specific_account_set_id)
        .await?
        .unwrap();
    assert_eq!(module_specific_account.children_ids, vec![child.id]);

    Ok(())
}
