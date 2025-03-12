use crate::{
    accounting_init::{constants::*, *},
    new_chart_of_accounts::NewChartOfAccounts,
};

use rbac_types::Subject;

pub(crate) async fn init(
    new_chart_of_accounts: &NewChartOfAccounts,
) -> Result<(), AccountingInitError> {
    create_new_chart_of_accounts(new_chart_of_accounts).await?;

    Ok(())
}

async fn create_new_chart_of_accounts(
    chart_of_accounts: &NewChartOfAccounts,
) -> Result<(), AccountingInitError> {
    if chart_of_accounts
        .find_by_reference(&Subject::System, CHART_REF.to_string())
        .await?
        .is_none()
    {
        chart_of_accounts
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
            )
            .await?;
    }

    Ok(())
}
