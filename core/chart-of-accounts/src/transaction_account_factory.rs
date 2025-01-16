use audit::AuditInfo;
use cala_ledger::{account::*, CalaLedger, LedgerOperation};

use crate::{
    chart_of_accounts::ChartRepo,
    error::CoreChartOfAccountsError,
    path::ControlSubAccountPath,
    primitives::{ChartAccountDetails, ChartCreationDetails, ChartId, LedgerAccountId},
};

#[derive(Clone)]
pub struct TransactionAccountFactory {
    repo: ChartRepo,
    cala: CalaLedger,
    chart_id: ChartId,
    control_sub_account: ControlSubAccountPath,
}

impl TransactionAccountFactory {
    pub(super) fn new(
        repo: &ChartRepo,
        cala: &CalaLedger,
        chart_id: ChartId,
        control_sub_account: ControlSubAccountPath,
    ) -> Self {
        Self {
            repo: repo.clone(),
            cala: cala.clone(),
            chart_id,
            control_sub_account,
        }
    }

    pub async fn create_transaction_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        account_id: impl Into<LedgerAccountId>,
        name: &str,
        description: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartAccountDetails, CoreChartOfAccountsError> {
        let mut chart = self
            .repo
            .find_by_id_in_tx(op.op().tx(), self.chart_id)
            .await?;

        let account_details = chart.add_transaction_account(
            ChartCreationDetails {
                account_id: account_id.into(),
                control_sub_account: self.control_sub_account,
                name: name.to_string(),
                description: description.to_string(),
            },
            audit_info,
        )?;

        self.repo.update_in_op(op.op(), &mut chart).await?;

        let new_account = NewAccount::builder()
            .id(account_details.account_id)
            .name(account_details.name.to_string())
            .description(account_details.description.to_string())
            .code(account_details.encoded_path.to_string())
            .normal_balance_type(account_details.path.normal_balance_type())
            .build()
            .expect("Could not build new account");

        self.cala.accounts().create_in_op(op, new_account).await?;

        Ok(account_details)
    }
}
