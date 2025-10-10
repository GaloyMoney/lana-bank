mod entity;
mod ledger;
mod primitives;
mod repo;

pub mod error;

use chrono::Utc;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{CalaLedger, JournalId};
use ledger::{AnnualClosingTransactionLedger, AnnualClosingTransactionParams, EntryParams};

use crate::{
    chart_of_accounts::ChartOfAccounts,
    primitives::{
        AnnualClosingTransactionId, CalaTxId, ChartId, CoreAccountingAction, CoreAccountingObject,
    },
};

use error::*;
use repo::*;

pub use entity::AnnualClosingTransaction;
#[cfg(feature = "json-schema")]
pub use entity::AnnualClosingTransactionEvent;
pub(super) use entity::*;
pub use repo::annual_closing_transaction_cursor::AnnualClosingTransactionsByCreatedAtCursor;

#[derive(Clone)]
pub struct AnnualClosingTransactions<Perms>
where
    Perms: PermissionCheck,
{
    ledger: AnnualClosingTransactionLedger,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal_id: JournalId,
    repo: AnnualClosingTransactionRepo,
}

impl<Perms> AnnualClosingTransactions<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        chart_of_accounts: &ChartOfAccounts<Perms>,
        cala: &CalaLedger,
        journal_id: JournalId,
    ) -> Self {
        let repo = AnnualClosingTransactionRepo::new(pool);
        Self {
            ledger: AnnualClosingTransactionLedger::new(cala),
            authz: authz.clone(),
            chart_of_accounts: chart_of_accounts.clone(),
            journal_id,
            repo,
        }
    }

    pub async fn execute(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_id: ChartId,
        reference: Option<String>,
        description: String,
    ) -> Result<AnnualClosingTransaction, AnnualClosingTransactionError> {
        // TODO: authz permissions (follow ManualTransactions).
        let effective = Utc::now();
        let ledger_tx_id: CalaTxId = CalaTxId::new();
        let closing_tx_id: AnnualClosingTransactionId = AnnualClosingTransactionId::new();

        let new_tx = NewAnnualClosingTransaction::builder()
            .id(closing_tx_id)
            .ledger_transaction_id(ledger_tx_id)
            .description(description.clone())
            .reference(reference)
            .build()
            .expect("Couldn't build new annual closing transaction");

        let mut db = self.repo.begin_op().await?;
        let annual_closing_transaction = self.repo.create_in_op(&mut db, new_tx).await?;
        let entries = self
            .chart_of_accounts
            .create_annual_closing_entries(effective, chart_id)
            .await?;

        let entry_params = entries.into_iter().map(EntryParams::from).collect();
        self.ledger
            .execute(
                db,
                ledger_tx_id,
                AnnualClosingTransactionParams {
                    journal_id: self.journal_id,
                    description,
                    entry_params,
                    effective: effective.date_naive(),
                },
            )
            .await?;

        Ok(annual_closing_transaction)
    }
}
