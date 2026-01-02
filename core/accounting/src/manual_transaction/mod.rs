mod entity;
pub mod error;
pub mod ledger;
mod primitives;
mod repo;

use tracing::instrument;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{CalaLedger, JournalId};
use ledger::{EntryParams, ManualTransactionLedger, ManualTransactionParams};
use tracing_macros::record_error_severity;

use crate::{
    chart_of_accounts::ChartOfAccounts,
    primitives::{
        CalaTxId, CoreAccountingAction, CoreAccountingObject, LedgerTransactionInitiator,
        ManualTransactionId,
    },
};
use error::*;

pub use entity::ManualTransaction;
#[cfg(feature = "json-schema")]
pub use entity::ManualTransactionEvent;
pub(super) use entity::*;
pub use primitives::*;
pub use repo::manual_transaction_cursor::ManualTransactionsByCreatedAtCursor;
use repo::*;

#[derive(Clone)]
pub struct ManualTransactions<Perms>
where
    Perms: PermissionCheck,
{
    ledger: ManualTransactionLedger,
    authz: Perms,
    chart_of_accounts: ChartOfAccounts<Perms>,
    journal_id: JournalId,
    repo: ManualTransactionRepo,
}

impl<Perms> ManualTransactions<Perms>
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
        let repo = ManualTransactionRepo::new(pool);
        Self {
            ledger: ManualTransactionLedger::new(cala),
            chart_of_accounts: chart_of_accounts.clone(),
            authz: authz.clone(),
            journal_id,
            repo,
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.manual_transaction.find_by_id", skip(self))]
    pub async fn find_manual_transaction_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ManualTransactionId> + std::fmt::Debug,
    ) -> Result<Option<ManualTransaction>, ManualTransactionError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::manual_transaction(id),
                CoreAccountingAction::MANUAL_TRANSACTION_LIST,
            )
            .await?;

        self.repo.maybe_find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.manual_transaction.list", skip(self))]
    pub async fn list_manual_transactions(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<ManualTransactionsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<ManualTransaction, ManualTransactionsByCreatedAtCursor>,
        ManualTransactionError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_manual_transactions(),
                CoreAccountingAction::MANUAL_TRANSACTION_LIST,
            )
            .await?;

        self.repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.manual_transaction.find_all", skip(self))]
    pub async fn find_all<T: From<ManualTransaction>>(
        &self,
        ids: &[ManualTransactionId],
    ) -> Result<HashMap<ManualTransactionId, T>, ManualTransactionError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "manual_transaction.execute", skip(self, entries), fields(subject = %sub, chart_ref = %chart_ref, effective = %effective, entries_count = entries.len()))]
    pub async fn execute(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart_ref: &str,
        reference: Option<String>,
        description: String,
        effective: chrono::NaiveDate,
        entries: Vec<ManualEntryInput>,
    ) -> Result<ManualTransaction, ManualTransactionError>
    where
        for<'a> LedgerTransactionInitiator:
            From<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_manual_transactions(),
                CoreAccountingAction::MANUAL_TRANSACTION_CREATE,
            )
            .await?;

        let ledger_tx_id = CalaTxId::new();
        let manual_tx_id = ManualTransactionId::new();
        let initiated_by = LedgerTransactionInitiator::from(sub);

        let new_tx = NewManualTransaction::builder()
            .id(manual_tx_id)
            .ledger_transaction_id(ledger_tx_id)
            .description(description.clone())
            .reference(reference)
            .build()
            .expect("Couldn't build new manual transaction");

        let mut db = self.repo.begin_op().await?;
        let manual_transaction = self.repo.create_in_op(&mut db, new_tx).await?;

        let mut entry_params = vec![];
        for e in entries {
            let account_id = self
                .chart_of_accounts
                .manual_transaction_account_id_for_account_id_or_code(
                    sub,
                    chart_ref,
                    e.account_id_or_code,
                )
                .await?;
            entry_params.push(EntryParams {
                account_id: account_id.into(),
                amount: e.amount,
                currency: e.currency,
                direction: e.direction,
                description: e.description,
            });
        }

        self.ledger
            .execute(
                &mut db,
                ledger_tx_id,
                ManualTransactionParams {
                    journal_id: self.journal_id,
                    description,
                    entry_params,
                    effective,
                    initiated_by,
                },
            )
            .await?;

        db.commit().await?;

        Ok(manual_transaction)
    }
}
