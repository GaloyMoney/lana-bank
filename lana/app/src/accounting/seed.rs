use chart_of_accounts::{
    error::CoreChartOfAccountError, CategoryPath, ChartId, ChartOfAccountCode,
};

use crate::chart_of_accounts::ChartOfAccounts;

use super::AccountingValues;

const CHART_REF: &str = "primary-chart";

const DEPOSITS_CONTROL_ACCOUNT_REF: &str = "deposits";
const DEPOSITS_CONTROL_ACCOUNT_NAME: &str = "Deposits";

const DEPOSITS_CONTROL_SUB_ACCOUNT_REF: &str = "deposits-user";
const DEPOSITS_CONTROL_SUB_ACCOUNT_NAME: &str = "User Deposits";

pub(super) async fn execute(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<AccountingValues, CoreChartOfAccountError> {
    let chart = match chart_of_accounts
        .find_by_reference(CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(ChartId::new(), CHART_REF.to_string())
                .await?
        }
    };

    let deposits_control_path = match chart_of_accounts
        .find_control_account_by_reference(chart.id, CHART_REF.to_string())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_account(
                    chart.id,
                    ChartOfAccountCode::Category(CategoryPath::Liabilities),
                    DEPOSITS_CONTROL_ACCOUNT_REF.to_string(),
                    DEPOSITS_CONTROL_ACCOUNT_NAME.to_string(),
                )
                .await?
        }
    };

    let deposits_control_sub_path = match chart_of_accounts
        .find_control_sub_account_by_reference(chart.id, CHART_REF.to_string())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_sub_account(
                    chart.id,
                    deposits_control_path,
                    DEPOSITS_CONTROL_SUB_ACCOUNT_NAME.to_string(),
                    DEPOSITS_CONTROL_SUB_ACCOUNT_REF.to_string(),
                )
                .await?
        }
    };

    Ok(AccountingValues {
        id: chart.id,
        deposits_control_sub_path,
    })
}
