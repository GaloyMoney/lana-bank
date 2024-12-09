pub mod error;
mod templates;

use cala_ledger::{account::*, CalaLedger};

use error::*;

#[derive(Clone)]
pub struct DepositLedger {
    cala: CalaLedger,
}

impl DepositLedger {
    pub async fn init(cala: &CalaLedger) -> Result<Self, DepositLedgerError> {
        templates::RecordDeposit::init(&cala).await?;
        Ok(Self { cala: cala.clone() })
    }

    pub async fn create_account_for_deposit_account(
        &self,
        op: es_entity::DbOp<'_>,
        id: impl Into<AccountId>,
        code: String,
    ) -> Result<(), DepositLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let new_account = NewAccount::builder()
            .id(id)
            .name("Deposit Account")
            .code(code)
            .build()
            .expect("Could not build new account");

        self.cala
            .accounts()
            .create_in_op(&mut op, new_account)
            .await?;

        op.commit().await?;

        Ok(())
    }
}
