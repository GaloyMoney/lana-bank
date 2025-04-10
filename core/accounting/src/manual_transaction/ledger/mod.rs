mod template;

use cala_ledger::{AccountId, AccountSetId, CalaLedger, account::NewAccount};

use crate::{
    Chart,
    primitives::{AccountCode, CalaTxId},
};

use super::{error::ManualTransactionError, primitives::AccountIdOrCode};

use template::*;
pub use template::{EntryParams, ManualTransactionParams};

#[derive(Clone)]
pub struct ManualTransactionLedger {
    cala: CalaLedger,
}

impl ManualTransactionLedger {
    pub fn new(cala: &CalaLedger) -> Self {
        Self { cala: cala.clone() }
    }

    pub async fn execute(
        &self,
        op: es_entity::DbOp<'_>,
        tx_id: CalaTxId,
        params: ManualTransactionParams,
    ) -> Result<(), ManualTransactionError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let template =
            ManualTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;

        self.cala
            .post_transaction_in_op(&mut op, tx_id, &template.code(), params)
            .await?;

        op.commit().await?;

        Ok(())
    }

    pub async fn resolve_account_id(
        &self,
        chart: &Chart,
        account_id_or_code: &AccountIdOrCode,
    ) -> Result<AccountId, ManualTransactionError> {
        match account_id_or_code {
            AccountIdOrCode::Id(account_id) => Ok((*account_id).into()),
            AccountIdOrCode::Code(code) => match chart.account_spec(code) {
                Some((_, parent_id)) => {
                    self.find_or_create_manual_account(
                        parent_id,
                        code,
                        code.manual_account_external_id(chart.id),
                    )
                    .await
                }
                None => Err(ManualTransactionError::UnknownAccountCode(code.to_string())),
            },
        }
    }

    async fn find_or_create_manual_account(
        &self,
        parent_id: &AccountSetId,
        parent_code: &AccountCode,
        external_id: String,
    ) -> Result<AccountId, ManualTransactionError> {
        let manual_account = self
            .cala
            .accounts()
            .find_by_external_id(external_id.clone())
            .await;

        match manual_account {
            Ok(existing) => Ok(existing.id()),
            Err(e) if e.was_not_found() => {
                self.create_manual_account(parent_id, parent_code, &external_id)
                    .await
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn create_manual_account(
        &self,
        parent_id: &AccountSetId,
        parent_code: &AccountCode,
        external_id: &str,
    ) -> Result<AccountId, ManualTransactionError> {
        let manual_account = self
            .cala
            .accounts()
            .create(
                NewAccount::builder()
                    .name(format!("{} Manual", parent_code))
                    .id(AccountId::new())
                    .code(external_id)
                    .external_id(external_id)
                    .build()
                    .unwrap(),
            )
            .await?;

        self.cala
            .account_sets()
            .add_member(*parent_id, manual_account.id)
            .await?;

        Ok(manual_account.id)
    }
}
