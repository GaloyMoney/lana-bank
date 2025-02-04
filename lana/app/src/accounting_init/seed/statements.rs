use crate::{accounting_init::*, balance_sheet::BalanceSheets};

use constants::{
    BALANCE_SHEET_NAME, OBS_BALANCE_SHEET_NAME, OBS_TRIAL_BALANCE_STATEMENT_NAME,
    PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
};

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
    let _primary_id = trial_balances
        .find_or_create_trial_balance_statement(TRIAL_BALANCE_STATEMENT_NAME.to_string())
        .await?;

    let _off_balance_sheet_id = trial_balances
        .find_or_create_trial_balance_statement(OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string())
        .await?;

    Ok(())
}

async fn create_pl_statements(
    pl_statements: &ProfitAndLossStatements,
) -> Result<(), AccountingInitError> {
    let _primary_id = pl_statements
        .find_or_create_pl_statement(PROFIT_AND_LOSS_STATEMENT_NAME.to_string())
        .await?;

    Ok(())
}

async fn create_balance_sheets(balance_sheets: &BalanceSheets) -> Result<(), AccountingInitError> {
    let _primary_id = match balance_sheets
        .find_by_name(BALANCE_SHEET_NAME.to_string())
        .await?
    {
        Some(balance_sheet_id) => balance_sheet_id,
        None => {
            balance_sheets
                .create_balance_sheet(LedgerAccountSetId::new(), BALANCE_SHEET_NAME.to_string())
                .await?
        }
    };

    let _off_balance_sheet_id = match balance_sheets
        .find_by_name(OBS_BALANCE_SHEET_NAME.to_string())
        .await?
    {
        Some(balance_sheet_id) => balance_sheet_id,
        None => {
            balance_sheets
                .create_balance_sheet(
                    LedgerAccountSetId::new(),
                    OBS_BALANCE_SHEET_NAME.to_string(),
                )
                .await?
        }
    };
    Ok(())
}
