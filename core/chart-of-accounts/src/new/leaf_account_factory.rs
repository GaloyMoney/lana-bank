use cala_ledger::{account::NewAccount, CalaLedger, LedgerOperation};

use super::{
    primitives::{AccountCode, ChartId},
    ChartRepo,
};
use crate::{error::CoreChartOfAccountsError, primitives::LedgerAccountId};

#[derive(Clone)]
pub struct LeafAccountFactory {
    cala: CalaLedger,
    repo: ChartRepo,
}

impl LeafAccountFactory {
    pub(super) fn new(cala: &CalaLedger, repo: &ChartRepo) -> Self {
        Self {
            cala: cala.clone(),
            repo: repo.clone(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_leaf_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        chart_id: ChartId,
        parent_code: AccountCode,
        account_id: impl Into<LedgerAccountId>,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<(), CoreChartOfAccountsError> {
        let account_id = account_id.into();

        let chart = self.repo.find_by_id(chart_id).await?;
        let (spec, account_set_id) = chart.account_spec(&parent_code).ok_or(
            CoreChartOfAccountsError::AccountNotFoundInChart(parent_code),
        )?;
        let new_account = NewAccount::builder()
            .id(account_id)
            .external_id(reference)
            .name(name.to_string())
            .description(description.to_string())
            .code(spec.leaf_account_code(chart_id, account_id))
            // .normal_balance_type(spec.normal_balance_type)
            .build()
            .expect("Could not build new account");

        let account = self.cala.accounts().create_in_op(op, new_account).await?;

        self.cala
            .account_sets()
            .add_member_in_op(op, *account_set_id, account.id)
            .await?;
        Ok(())
    }
}
