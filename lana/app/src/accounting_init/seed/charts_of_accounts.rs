use crate::accounting_init::{constants::*, *};

use core_accounting::AccountingBaseConfig;
use rbac_types::Subject;

use super::module_config::{chart_integration_config::*, credit::*, deposit::*};

pub(crate) async fn init(
    accounting: &Accounting,
    credit: &Credit,
    deposit: &Deposits,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    create_chart_of_accounts(accounting, accounting_init_config.clone()).await?;

    seed_chart_of_accounts(accounting, credit, deposit, accounting_init_config).await?;

    Ok(())
}

async fn create_chart_of_accounts(
    accounting: &Accounting,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let opening_date = accounting_init_config
        .chart_of_accounts_opening_date
        .ok_or_else(|| {
            AccountingInitError::MissingConfig("chart_of_accounts_opening_date".to_string())
        })?;
    if accounting
        .chart_of_accounts()
        .maybe_find_by_reference(CHART_REF)
        .await?
        .is_none()
    {
        let chart = accounting
            .chart_of_accounts()
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
            )
            .await?;

        accounting
            .fiscal_year()
            .init_for_chart(&Subject::System, opening_date, chart.id)
            .await?;
    }

    Ok(())
}

async fn seed_chart_of_accounts(
    accounting: &Accounting,
    credit: &Credit,
    deposit: &Deposits,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let AccountingInitConfig {
        chart_of_accounts_seed_path: seed_path,
        chart_of_accounts_integration_config_path: chart_integration_config_path,
        credit_config_path,
        deposit_config_path,
        chart_of_accounts_opening_date: _,
    } = accounting_init_config;

    let data = match seed_path {
        Some(seed_path) => std::fs::read_to_string(seed_path)?,
        None => return Ok(()),
    };

    let accounting_integration_config: AccountingBaseConfig = match chart_integration_config_path {
        Some(config_path) => load_chart_integration_config_from_path(config_path)?,
        None => return Ok(()),
    };

    let chart = accounting
        .import_csv_with_base_config(
            &Subject::System,
            CHART_REF,
            data,
            accounting_integration_config,
            BALANCE_SHEET_NAME,
            PROFIT_AND_LOSS_STATEMENT_NAME,
            TRIAL_BALANCE_STATEMENT_NAME,
        )
        .await?;

    if let Some(config_path) = credit_config_path {
        credit_module_configure(credit, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    if let Some(config_path) = deposit_config_path {
        deposit_module_configure(deposit, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    Ok(())
}
