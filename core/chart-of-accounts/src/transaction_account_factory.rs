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

    async fn create_transaction_account(
        &self,
        account_id: impl Into<LedgerAccountId>,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<(), CoreChartOfAccountsError> {
        let mut op = self.cala.begin_operation().await?;
        self.create_transaction_account_in_op(&mut op, account_id, reference, name, description)
            .await?;
        op.commit().await?;

        Ok(())
    }

    pub async fn find_or_create_transaction_account(
        &self,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<LedgerAccountId, CoreChartOfAccountsError> {
        match self
            .cala
            .accounts()
            .find_by_external_id(reference.to_string())
            .await
        {
            Ok(account) => return Ok(account.id),
            Err(e) if e.was_not_found() => (),
            Err(e) => return Err(e.into()),
        };

        let id = LedgerAccountId::new();
        self.create_transaction_account(id, reference, name, description)
            .await?;

        Ok(id)
    }

    pub async fn create_transaction_account_in_op(
        &self,
        op: &mut LedgerOperation<'_>,
        account_id: impl Into<LedgerAccountId>,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<(), CoreChartOfAccountsError> {
        let account_id = account_id.into();

        let new_account = NewAccount::builder()
            .id(account_id)
            .external_id(reference.to_string())
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
