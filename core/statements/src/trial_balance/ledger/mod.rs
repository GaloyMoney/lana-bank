pub mod error;

use cala_ledger::{account_set::NewAccountSet, CalaLedger, DebitOrCredit, JournalId};

use crate::primitives::LedgerAccountSetId;

use error::*;

#[derive(Clone)]
pub struct TrialBalanceStatementLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl TrialBalanceStatementLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    pub async fn create(
        &self,
        op: es_entity::DbOp<'_>,
        statement_id: impl Into<LedgerAccountSetId>,
        name: &str,
    ) -> Result<(), TrialBalanceStatementLedgerError> {
        let mut op = self.cala.ledger_operation_from_db_op(op);

        let new_account_set = NewAccountSet::builder()
            .id(statement_id)
            .journal_id(self.journal_id)
            .name(name)
            .description(name)
            .normal_balance_type(DebitOrCredit::Debit)
            .build()
            .expect("Could not build new account set");
        self.cala
            .account_sets()
            .create_in_op(&mut op, new_account_set)
            .await?;

        op.commit().await?;
        Ok(())
    }

    pub async fn add_member(
        &self,
        op: es_entity::DbOp<'_>,
        statement_id: impl Into<LedgerAccountSetId>,
        member: LedgerAccountSetId,
    ) -> Result<(), TrialBalanceStatementLedgerError> {
        let statement_id = statement_id.into();

        let mut op = self.cala.ledger_operation_from_db_op(op);
        self.cala
            .account_sets()
            .add_member_in_op(&mut op, statement_id, member)
            .await?;

        op.commit().await?;
        Ok(())
    }
}
