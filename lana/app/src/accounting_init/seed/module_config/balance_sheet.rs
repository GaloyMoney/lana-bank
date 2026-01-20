use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::BALANCE_SHEET_NAME},
    balance_sheet::BalanceSheets,
};

use rbac_types::Subject;

pub(in crate::accounting_init::seed) async fn balance_sheet_module_configure(
    balance_sheet: &BalanceSheets,
    chart: &Chart,
) -> Result<(), AccountingInitError> {
    balance_sheet
        .link_chart_account_sets(&Subject::System, BALANCE_SHEET_NAME.to_string(), chart)
        .await?;
    Ok(())
}
