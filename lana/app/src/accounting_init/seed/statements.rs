use constants::{
    OBS_TRIAL_BALANCE_STATEMENT_NAME, PROFIT_AND_LOSS_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_NAME,
};

use crate::accounting_init::*;

pub(crate) async fn init(
    trial_balances: &TrialBalances,
    pl_statements: &ProfitAndLossStatements,
) -> Result<StatementsInit, AccountingInitError> {
    create_trial_balances(trial_balances).await?;

    create_pl_statements(pl_statements).await?;

    Ok(StatementsInit)
}

async fn create_trial_balances(trial_balances: &TrialBalances) -> Result<(), AccountingInitError> {
    let _primary_id = match trial_balances
        .find_by_name(TRIAL_BALANCE_STATEMENT_NAME.to_string())
        .await?
    {
        Some(trial_balance_id) => trial_balance_id,
        None => {
            trial_balances
                .create_trial_balance_statement(
                    TrialBalanceId::new(),
                    TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                )
                .await?
        }
    };

    let _off_balance_sheet_id = match trial_balances
        .find_by_name(OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            trial_balances
                .create_trial_balance_statement(
                    TrialBalanceId::new(),
                    OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                )
                .await?
        }
    };

    Ok(())
}

async fn create_pl_statements(
    pl_statements: &ProfitAndLossStatements,
) -> Result<(), AccountingInitError> {
    let _primary_id = match pl_statements
        .find_by_name(PROFIT_AND_LOSS_STATEMENT_NAME.to_string())
        .await?
    {
        Some(pl_statement_id) => pl_statement_id,
        None => {
            pl_statements
                .create_pl_statement(
                    ProfitAndLossStatementId::new(),
                    PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
                )
                .await?
        }
    };

    Ok(())
}
