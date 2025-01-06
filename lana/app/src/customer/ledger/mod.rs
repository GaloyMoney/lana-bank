mod accounts;
pub mod error;

use cala_ledger::{account::NewAccount, CalaLedger};

use crate::primitives::CustomerId;

pub use accounts::*;
use error::*;

#[derive(Clone)]
pub struct CustomerLedger {
    cala: CalaLedger,
}

impl CustomerLedger {
    pub async fn init(cala: &CalaLedger) -> Result<Self, CustomerLedgerError> {
        Ok(Self { cala: cala.clone() })
    }

    pub async fn create_accounts_for_customer(
        &self,
        op: es_entity::DbOp<'_>,
        customer_id: CustomerId,
        CustomerAccountIds { deposit_account_id }: CustomerAccountIds,
    ) -> Result<(), CustomerLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);
        let new_accounts = vec![NewAccount::builder()
            .id(deposit_account_id)
            .name(format!("Customer Checking Account for {}", customer_id))
            .code(format!("CUSTOMERS.CHECKING.{}", customer_id))
            .build()
            .expect("new account")];

        self.cala
            .accounts()
            .create_all_in_op(&mut op, new_accounts)
            .await?;

        op.commit().await?;

        Ok(())
    }
}
