use std::path::PathBuf;

use crate::{
    accounting::{ChartId, ChartOfAccounts},
    accounting_init::{constants::*, *},
};

use rbac_types::Subject;

pub(crate) async fn init(
    chart_of_accounts: &ChartOfAccounts,
    seed_path: Option<PathBuf>,
) -> Result<(), AccountingInitError> {
    let chart_id = create_chart_of_accounts(chart_of_accounts).await?;

    if let Some(path) = seed_path {
        seed_chart_of_accounts(chart_of_accounts, chart_id, path).await?;
    }
    Ok(())
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartId, AccountingInitError> {
    if let Some(chart) = chart_of_accounts.find_by_reference(CHART_REF).await? {
        Ok(chart.id)
    } else {
        Ok(chart_of_accounts
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
            )
            .await?
            .id)
    }
}

async fn seed_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    chart_id: ChartId,
    seed_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = std::fs::read_to_string(seed_path)?;
    chart_of_accounts
        .import_from_csv(&Subject::System, chart_id, data)
        .await?;

    Ok(())
}
