use anyhow::Result;

use crate::cli::FinancialStatementAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: FinancialStatementAction,
    json: bool,
) -> Result<()> {
    match action {
        FinancialStatementAction::BalanceSheet { from, until } => {
            let vars = balance_sheet_get::Variables { from, until };
            let data = client.execute::<BalanceSheetGet>(vars).await?;
            if json {
                output::print_json(&data.balance_sheet)?;
            } else {
                output::print_json(&data.balance_sheet)?;
            }
        }
        FinancialStatementAction::TrialBalance { from, until } => {
            let vars = trial_balance_get::Variables { from, until };
            let data = client.execute::<TrialBalanceGet>(vars).await?;
            if json {
                output::print_json(&data.trial_balance)?;
            } else {
                output::print_json(&data.trial_balance)?;
            }
        }
        FinancialStatementAction::ProfitAndLoss { from, until } => {
            let vars = profit_and_loss_get::Variables { from, until };
            let data = client.execute::<ProfitAndLossGet>(vars).await?;
            if json {
                output::print_json(&data.profit_and_loss_statement)?;
            } else {
                output::print_json(&data.profit_and_loss_statement)?;
            }
        }
    }
    Ok(())
}
