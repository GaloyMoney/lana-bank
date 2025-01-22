use constants::{
    OBS_TRIAL_BALANCE_STATEMENT_NAME, OBS_TRIAL_BALANCE_STATEMENT_REF,
    TRIAL_BALANCE_STATEMENT_NAME, TRIAL_BALANCE_STATEMENT_REF,
};
use primitives::TrialBalanceStatementIds;
use statements::TrialBalanceStatementId;

use crate::accounting_init::*;

pub(crate) async fn init(statements: &Statements) -> Result<StatementsInit, AccountingInitError> {
    let trial_balance_ids = create_trial_balances(statements).await?;

    Ok(StatementsInit { trial_balance_ids })
}

async fn create_trial_balances(
    statements: &Statements,
) -> Result<TrialBalanceStatementIds, AccountingInitError> {
    let primary = match statements
        .find_by_reference(TRIAL_BALANCE_STATEMENT_REF.to_string())
        .await?
    {
        Some(statement) => statement,
        None => {
            statements
                .create_trial_balance_statement(
                    TrialBalanceStatementId::new(),
                    TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                    TRIAL_BALANCE_STATEMENT_REF.to_string(),
                )
                .await?
        }
    };

    let off_balance_sheet = match statements
        .find_by_reference(OBS_TRIAL_BALANCE_STATEMENT_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            statements
                .create_trial_balance_statement(
                    TrialBalanceStatementId::new(),
                    OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
                    OBS_TRIAL_BALANCE_STATEMENT_REF.to_string(),
                )
                .await?
        }
    };

    Ok(TrialBalanceStatementIds {
        primary: primary.id,
        off_balance_sheet: off_balance_sheet.id,
    })
}
