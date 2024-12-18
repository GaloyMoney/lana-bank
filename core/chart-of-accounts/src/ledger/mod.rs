pub mod error;

use cala_ledger::{account::*, CalaLedger, DebitOrCredit};

use crate::{
    code::{ChartOfAccountCategoryCode, ChartOfAccountCode},
    primitives::ChartOfAccountAccountDetails,
};

use error::*;

#[derive(Clone)]
pub struct ChartOfAccountLedger {
    cala: CalaLedger,
}

impl ChartOfAccountLedger {
    pub async fn init(cala: &CalaLedger) -> Result<Self, ChartOfAccountLedgerError> {
        Ok(Self { cala: cala.clone() })
    }

    fn normal_balance_type_for_code(code: ChartOfAccountCode) -> DebitOrCredit {
        match code.category() {
            ChartOfAccountCategoryCode::Assets | ChartOfAccountCategoryCode::Expenses => {
                DebitOrCredit::Debit
            }
            _ => DebitOrCredit::Credit,
        }
    }

    pub async fn create_transaction_account(
        &self,
        op: es_entity::DbOp<'_>,
        account_details: &ChartOfAccountAccountDetails,
    ) -> Result<(), ChartOfAccountLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let new_account = NewAccount::builder()
            .id(account_details.account_id)
            .name(account_details.name.to_string())
            .description(account_details.description.to_string())
            .code(account_details.code.to_string())
            .normal_balance_type(Self::normal_balance_type_for_code(account_details.code))
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
