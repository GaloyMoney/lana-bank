pub mod error;
mod ledger;
mod primitives;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;

use crate::primitives::{
    CalaAccountBalance, CalaAccountSetId, CalaJournalId, CoreAccountingAction,
    CoreAccountingObject, LedgerAccountId,
};

use error::*;
use ledger::*;
pub use primitives::*;

pub struct LedgerAccount {
    pub id: LedgerAccountId,
    pub usd_balance: Option<CalaAccountBalance>,
    pub btc_balance: Option<CalaAccountBalance>,
}

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
    pub fn new(authz: &Perms, cala: &CalaLedger, journal_id: CalaJournalId) -> Self {
        Self {
            authz: authz.clone(),
            ledger: LedgerAccountLedger::new(cala, journal_id),
        }
    }

    pub async fn balance<T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CalaAccountSetId>,
    ) -> Result<T, LedgerAccountError>
    where
        T: From<Option<cala_ledger::balance::AccountBalance>>,
    {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_account(id.into()),
                CoreAccountingAction::LEDGER_ACCOUNT_READ,
            )
            .await?;

        let res = self.ledger.balance(id).await?;

        Ok(res)
    }

    pub async fn history(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CalaAccountSetId>,
        args: es_entity::PaginatedQueryArgs<LedgerAccountHistoryCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerAccountEntry, LedgerAccountHistoryCursor>,
        LedgerAccountError,
    > {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_account(id.into()),
                CoreAccountingAction::LEDGER_ACCOUNT_READ_HISTORY,
            )
            .await?;

        let res = self
            .ledger
            .account_set_history::<LedgerAccountEntry, LedgerAccountHistoryCursor>(id.into(), args)
            .await?;

        Ok(res)
    }

    #[instrument(name = "accounting.ledger_account.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<LedgerAccount>, LedgerAccountError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::ledger_account(id),
                CoreAccountingAction::LEDGER_ACCOUNT_READ,
            )
            .await?;
        let mut accounts = self.ledger.load_ledger_accounts([id].as_ref()).await?;
        Ok(accounts.remove(&id))
    }

    pub async fn find_all<T: From<LedgerAccount>>(
        &self,
        ids: &[LedgerAccountId],
    ) -> Result<HashMap<LedgerAccountId, T>, LedgerAccountError> {
        Ok(self.ledger.load_ledger_accounts(ids).await?)
    }
}
