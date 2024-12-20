use audit::AuditInfo;
use cala_ledger::{account::*, CalaLedger};

use crate::{
    chart_of_accounts::ChartOfAccountRepo,
    code::ChartOfAccountCode,
    error::CoreChartOfAccountError,
    primitives::{ChartId, ChartOfAccountAccountDetails, LedgerAccountId},
};

#[derive(Clone)]
pub struct TransactionAccountFactory {
    repo: ChartOfAccountRepo,
    cala: CalaLedger,
    chart_id: ChartId,
    control_sub_account: ChartOfAccountCode,
}

impl TransactionAccountFactory {
    pub(super) fn new(
        repo: &ChartOfAccountRepo,
        cala: &CalaLedger,
        chart_id: ChartId,
        control_sub_account: ChartOfAccountCode,
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
        mut op: es_entity::DbOp<'_>,
        account_id: impl Into<LedgerAccountId>,
        name: &str,
        description: &str,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountAccountDetails, CoreChartOfAccountError> {
        let mut chart = self.repo.find_by_id(self.chart_id).await?;

        let account_details = chart.add_transaction_account(
            account_id,
            self.control_sub_account,
            name,
            description,
            audit_info,
        )?;

        self.repo.update_in_op(&mut op, &mut chart).await?;

        let mut op = self.cala.ledger_operation_from_db_op(op);

        let new_account = NewAccount::builder()
            .id(account_details.account_id)
            .name(account_details.name.to_string())
            .description(account_details.description.to_string())
            .code(account_details.code.to_string())
            .normal_balance_type(account_details.path.normal_balance_type())
            .build()
            .expect("Could not build new account");

        self.cala
            .accounts()
            .create_in_op(&mut op, new_account)
            .await?;

        op.commit().await?;

        Ok(account_details)
    }
}
