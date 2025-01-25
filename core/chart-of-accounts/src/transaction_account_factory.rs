use cala_ledger::{account::*, CalaLedger, LedgerOperation};

use crate::{error::CoreChartOfAccountsError, primitives::LedgerAccountId};

use super::ControlSubAccountDetails;

#[derive(Clone)]
pub struct TransactionAccountFactory {
    cala: CalaLedger,
    pub control_sub_account: ControlSubAccountDetails,
}

impl TransactionAccountFactory {
    pub(super) fn new(cala: &CalaLedger, control_sub_account: ControlSubAccountDetails) -> Self {
        Self {
            cala: cala.clone(),
            control_sub_account,
        }
    }

    pub async fn create_transaction_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        account_id: impl Into<LedgerAccountId>,
        name: &str,
        description: &str,
    ) -> Result<(), CoreChartOfAccountsError> {
        let account_id = account_id.into();

        let new_account = NewAccount::builder()
            .id(account_id)
            .name(name.to_string())
            .description(description.to_string())
            .code(format!("{}.{}", self.control_sub_account.path, account_id))
            .normal_balance_type(self.control_sub_account.path.normal_balance_type())
            .build()
            .expect("Could not build new account");

        let account = self.cala.accounts().create_in_op(op, new_account).await?;

        self.cala
            .account_sets()
            .add_member_in_op(op, self.control_sub_account.account_set_id, account.id)
            .await?;

        Ok(())
    }
}
