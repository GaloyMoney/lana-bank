use crate::accounting_init::*;

use constants::{BALANCE_SHEET_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME};

pub(crate) async fn init(
    trial_balances: &TrialBalances,
    pl_statements: &ProfitAndLossStatements,
    balance_sheets: &BalanceSheets,
) -> Result<StatementsInit, AccountingInitError> {
    create_trial_balances(trial_balances).await?;

    create_pl_statements(pl_statements).await?;

    create_balance_sheets(balance_sheets).await?;

    Ok(StatementsInit)
}

async fn create_trial_balances(trial_balances: &TrialBalances) -> Result<(), AccountingInitError> {
    trial_balances
        .create_trial_balance_statement(TRIAL_BALANCE_STATEMENT_NAME.to_string())
        .await?;

    Ok(())
}

async fn create_pl_statements(
    pl_statements: &ProfitAndLossStatements,
) -> Result<(), AccountingInitError> {
    pl_statements
        .create_pl_statement(PROFIT_AND_LOSS_STATEMENT_NAME.to_string())
        .await?;

    Ok(())
}

async fn create_balance_sheets(balance_sheets: &BalanceSheets) -> Result<(), AccountingInitError> {
    balance_sheets
        .create_balance_sheet(BALANCE_SHEET_NAME.to_string())
        .await?;

    Ok(())
}
