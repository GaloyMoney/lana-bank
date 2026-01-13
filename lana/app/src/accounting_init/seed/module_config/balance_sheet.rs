use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::BALANCE_SHEET_NAME},
    balance_sheet::{BalanceSheets, error::BalanceSheetError},
};

use rbac_types::Subject;

pub(in crate::accounting_init::seed) async fn balance_sheet_module_configure(
    balance_sheet: &BalanceSheets,
    chart: &Chart,
) -> Result<(), AccountingInitError> {
    match balance_sheet
        .set_chart_of_accounts_integration_config(
            &Subject::System,
            BALANCE_SHEET_NAME.to_string(),
            chart,
        )
        .await
    {
        Ok(_) => (),
        Err(BalanceSheetError::BalanceSheetConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
