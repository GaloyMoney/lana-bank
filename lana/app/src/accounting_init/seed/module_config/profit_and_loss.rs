use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::PROFIT_AND_LOSS_STATEMENT_NAME},
    profit_and_loss::{
        ChartOfAccountsIntegrationConfig, ProfitAndLossStatements,
        error::ProfitAndLossStatementError,
    },
};

use rbac_types::Subject;

// TODO: Remove un-needed config.
#[derive(Deserialize)]
struct ProfitAndLossStatementConfigData {
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
}

pub(in crate::accounting_init::seed) async fn profit_and_loss_module_configure(
    profit_and_loss: &ProfitAndLossStatements,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    // TODO: Remove un-needed config.
    let data = fs::read_to_string(config_path)?;
    let ProfitAndLossStatementConfigData {
        revenue_code,
        cost_of_revenue_code,
        expenses_code,
    } = serde_json::from_str(&data)?;

    let _config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_revenue_code: revenue_code.parse()?,
        chart_of_accounts_cost_of_revenue_code: cost_of_revenue_code.parse()?,
        chart_of_accounts_expenses_code: expenses_code.parse()?,
    };

    match profit_and_loss
        .set_chart_of_accounts_integration_config(
            &Subject::System,
            PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
            chart,
        )
        .await
    {
        Ok(_) => (),
        Err(ProfitAndLossStatementError::ProfitAndLossStatementConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
