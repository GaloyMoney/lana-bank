pub mod error;
mod ledger;
mod primitives;

use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use rbac_types::{LedgerAccountAction, Subject};

use crate::{
    authorization::{Authorization, Object},
    primitives::{LedgerAccountSetId, LedgerJournalId},
};

use error::*;
use ledger::*;
pub use primitives::*;

#[derive(Clone)]
pub struct LedgerAccounts {
    authz: Authorization,
    ledger: LedgerAccountLedger,
}

impl LedgerAccounts {
    pub fn init(authz: &Authorization, cala: &CalaLedger, journal_id: LedgerJournalId) -> Self {
        Self {
            authz: authz.clone(),
            ledger: LedgerAccountLedger::init(cala, journal_id),
        }
    }

    pub async fn history(
        &self,
        sub: &Subject,
        id: impl Into<LedgerAccountSetId>,
        args: es_entity::PaginatedQueryArgs<LedgerAccountHistoryCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<LedgerAccountEntry, LedgerAccountHistoryCursor>,
        LedgerAccountError,
    > {
        self.authz
            .enforce_permission(sub, Object::LedgerAccount, LedgerAccountAction::ReadHistory)
            .await?;

        let res = self
            .ledger
            .account_set_history::<LedgerAccountEntry, LedgerAccountHistoryCursor>(id.into(), args)
            .await?;

        Ok(res)
    }
}
