use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::PROFIT_AND_LOSS_STATEMENT_NAME},
    profit_and_loss::{ProfitAndLossStatements, error::ProfitAndLossStatementError},
};

use rbac_types::Subject;

pub(in crate::accounting_init::seed) async fn profit_and_loss_module_configure(
    profit_and_loss: &ProfitAndLossStatements,
    chart: &Chart,
) -> Result<(), AccountingInitError> {
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
