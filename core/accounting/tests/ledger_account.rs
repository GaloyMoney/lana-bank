mod helpers;

use authz::dummy::DummySubject;
use cala_ledger::{CalaLedger, CalaLedgerConfig};
use core_accounting::CoreAccounting;
use helpers::{action, object};

#[tokio::test]
async fn ledger_account_ancestors() -> anyhow::Result<()> {
    use rand::Rng;
    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let journal_id = helpers::init_journal(&cala).await?;

    let accounting = CoreAccounting::new(&pool, &authz, &cala, journal_id);
    let chart_ref = format!("ref-{:08}", rand::thread_rng().gen_range(0..10000));
    let chart = accounting
        .chart_of_accounts()
        .create_chart(&DummySubject, "Test chart".to_string(), chart_ref.clone())
        .await?;
    let import = r#"
        1,,Root
        11,,Child
        11,1,Grandchild
        "#
    .to_string();
    let chart_id = chart.id;
    accounting
        .chart_of_accounts()
        .import_from_csv(&DummySubject, chart_id, import)
        .await?;

    let ledger_account = accounting
        .find_ledger_account_by_code(&DummySubject, &chart_ref, "11.1".to_string())
        .await?
        .expect("ledger account not found");

    assert_eq!(ledger_account.ancestor_ids.len(), 2);

    Ok(())
}
