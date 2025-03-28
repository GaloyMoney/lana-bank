pub mod error;
mod ledger;
mod primitives;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

use crate::primitives::{
    CoreAccountingAction, CoreAccountingObject, LedgerAccountSetId, LedgerJournalId,
};

use error::*;
use ledger::*;
pub use primitives::*;

#[derive(Clone)]
pub struct LedgerAccounts<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    ledger: LedgerAccountLedger,
}

impl<Perms> LedgerAccounts<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(authz: &Perms, cala: &CalaLedger, journal_id: LedgerJournalId) -> Self {
        Self {
            authz: authz.clone(),
            ledger: LedgerAccountLedger::new(cala, journal_id),
        }
    }

    pub async fn balance<T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerAccountSetId>,
    ) -> Result<T, LedgerAccountError>
    where
        T: From<Option<cala_ledger::balance::AccountBalance>>,
    {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::LedgerAccount,
                CoreAccountingAction::LEDGER_ACCOUNT_READ_BALANCE,
            )
            .await?;

        let res = self.ledger.balance(id.into()).await?;

        Ok(res)
    }

    pub async fn history(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerAccountSetId>,
        args: es_entity::PaginatedQueryArgs<LedgerAccountHistoryCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerAccountEntry, LedgerAccountHistoryCursor>,
        LedgerAccountError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::LedgerAccount,
                CoreAccountingAction::LEDGER_ACCOUNT_READ_HISTORY,
            )
            .await?;

        let res = self
            .ledger
            .account_set_history::<LedgerAccountEntry, LedgerAccountHistoryCursor>(id.into(), args)
            .await?;

        Ok(res)
    }
}
