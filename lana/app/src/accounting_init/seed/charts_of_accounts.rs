use crate::{
    accounting::ChartOfAccounts,
    accounting_init::{constants::*, *},
};

use rbac_types::Subject;

pub(crate) async fn init(chart_of_accounts: &ChartOfAccounts) -> Result<(), AccountingInitError> {
    create_chart_of_accounts(chart_of_accounts).await?;

    Ok(())
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<(), AccountingInitError> {
    if chart_of_accounts
        .find_by_reference(CHART_REF)
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
