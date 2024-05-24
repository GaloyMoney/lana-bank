use lava_core::ledger::*;

#[tokio::test]
async fn init() -> anyhow::Result<()> {
    let ledger = lava_core::ledger::Ledger::init(LedgerConfig::default()).await?;
    let account_id = ledger
        .cala
        .find_account_by_external_id("lava:loan-omnibus".to_string())
        .await;
    assert!(account_id.is_ok());
    Ok(())
}
