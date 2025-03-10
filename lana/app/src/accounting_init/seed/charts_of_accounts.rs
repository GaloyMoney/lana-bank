use crate::accounting_init::{constants::*, *};

pub(crate) async fn init(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartsInit, AccountingInitError> {
    let chart_ids = &create_charts_of_accounts(chart_of_accounts).await?;

    Ok(ChartsInit {
        chart_ids: *chart_ids,
    })
}

async fn create_charts_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartIds, AccountingInitError> {
    let primary = match chart_of_accounts
        .find_by_reference(CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(
                    ChartId::new(),
                    CHART_NAME.to_string(),
                    CHART_REF.to_string(),
                )
                .await?
        }
    };

    let off_balance_sheet = match chart_of_accounts
        .find_by_reference(OBS_CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(
                    ChartId::new(),
                    OBS_CHART_NAME.to_string(),
                    OBS_CHART_REF.to_string(),
                )
                .await?
        }
    };

    Ok(ChartIds {
        primary: primary.id,
        off_balance_sheet: off_balance_sheet.id,
    })
}
