use anyhow::Result;

use crate::cli::FinancialStatementAction;
use crate::client::GraphQLClient;
use crate::date::normalize_graphql_date;
use crate::graphql::*;
use crate::output;

pub async fn execute(
    client: &mut GraphQLClient,
    action: FinancialStatementAction,
    _json: bool,
) -> Result<()> {
    match action {
        FinancialStatementAction::BalanceSheet { as_of } => {
            let as_of = normalize_graphql_date(&as_of)?;
            let vars = balance_sheet_get::Variables { as_of };
            let data = client.execute::<BalanceSheetGet>(vars).await?;
            output::print_json(&data.balance_sheet)?;
        }
        FinancialStatementAction::TrialBalance { from, until } => {
            let from = normalize_graphql_date(&from)?;
            let until = normalize_graphql_date(&until)?;
            let vars = trial_balance_get::Variables { from, until };
            let data = client.execute::<TrialBalanceGet>(vars).await?;
            output::print_json(&data.trial_balance)?;
        }
        FinancialStatementAction::ProfitAndLoss { from, until } => {
            let from = normalize_graphql_date(&from)?;
            let until = until.as_deref().map(normalize_graphql_date).transpose()?;
            let vars = profit_and_loss_get::Variables { from, until };
            let data = client.execute::<ProfitAndLossGet>(vars).await?;
            output::print_json(&data.profit_and_loss_statement)?;
        }
    }
    Ok(())
}
