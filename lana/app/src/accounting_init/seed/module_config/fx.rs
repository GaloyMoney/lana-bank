use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    fx::{Fx, FxChartOfAccountsIntegrationConfig},
};

use rbac_types::Subject;

#[derive(Deserialize)]
struct FxConfigData {
    trading_account_parent_code: String,
    rounding_account_parent_code: String,
    realized_gain_parent_code: String,
    realized_loss_parent_code: String,
}

pub(in crate::accounting_init::seed) async fn fx_module_configure(
    fx: &Fx,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let FxConfigData {
        trading_account_parent_code,
        rounding_account_parent_code,
        realized_gain_parent_code,
        realized_loss_parent_code,
    } = serde_json::from_str(&data)?;

    let config_values = FxChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        trading_account_parent_code: trading_account_parent_code.parse()?,
        rounding_account_parent_code: rounding_account_parent_code.parse()?,
        realized_gain_parent_code: realized_gain_parent_code.parse()?,
        realized_loss_parent_code: realized_loss_parent_code.parse()?,
    };

    match fx
        .chart_of_accounts_integrations()
        .set_config(
            &Subject::System(audit::SystemActor::BOOTSTRAP),
            chart,
            config_values,
        )
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
