mod template;

use cala_ledger::{CalaLedger, JournalId};

use cala_ledger::{
    account_set::{error::AccountSetError, NewAccountSet},
    AccountSetId,
};

use super::{
    error::ManualTransactionError,
    primitives::{AccountRef, CalaTransactionId},
};
use crate::{Chart, LedgerAccountId};
use template::*;
pub use template::{EntryParams, ManualTransactionParams};

#[derive(Clone)]
pub struct ManualTransactionLedger {
    cala: CalaLedger,
    journal_id: JournalId,
}

impl ManualTransactionLedger {
    pub fn new(cala: &CalaLedger, journal_id: JournalId) -> Self {
        Self {
            cala: cala.clone(),
            journal_id,
        }
    }

    pub async fn execute(
        &self,
        _tx_id: impl Into<CalaTransactionId>,
        params: ManualTransactionParams,
    ) -> Result<(), ManualTransactionError> {
        let _ = ManualTransactionTemplate::init(&self.cala, params.entry_params.len()).await?;
        // self.post_transaction();
        Ok(())
    }

    /// Returns account ID representing `account_ref`.
    pub async fn resolve_account_ref(
        &self,
        chart: &Chart,
        account_ref: &AccountRef,
    ) -> Result<LedgerAccountId, ManualTransactionError> {
        match account_ref {
            AccountRef::Id(account_id) => Ok(*account_id),
            AccountRef::Code(code) => match chart.account_spec(code) {
                Some((_, coa_parent)) => {
                    self.find_or_create_manual_account_set(
                        coa_parent,
                        code.manual_account_external_id(chart.id),
                    )
                    .await
                }
                None => todo!("err"),
            },
        }
    }

    /// Returns account set for manual transactions with `external_id` if it exists,
    /// otherwise creates new one under `parent` (which is expected to be in the Chart of Accounts).
    /// Returns ID of the existing or new account set.
    async fn find_or_create_manual_account_set(
        &self,
        parent: &AccountSetId,
        external_id: String,
    ) -> Result<LedgerAccountId, ManualTransactionError> {
        let manual_account_set = self
            .cala
            .account_sets()
            .find_by_external_id(external_id.clone())
            .await;

        match manual_account_set {
            Ok(existing) => Ok(existing.id().into()),
            Err(AccountSetError::CouldNotFindByExternalId(_)) => {
                self.create_manual_account_set(parent, &external_id).await
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Creates new account set for manual transactions with `external_id`
    /// under `parent`. Returns ID of the new account set.
    async fn create_manual_account_set(
        &self,
        parent: &AccountSetId,
        external_id: &str,
    ) -> Result<LedgerAccountId, ManualTransactionError> {
        let manual_account_set = self
            .cala
            .account_sets()
            .create(
                NewAccountSet::builder()
                    .id(AccountSetId::new())
                    .external_id(external_id)
                    .name("???")
                    .journal_id(self.journal_id)
                    .build()
                    .unwrap(),
            )
            .await?;

        self.cala
            .account_sets()
            .add_member(*parent, manual_account_set.id)
            .await?;

        Ok(manual_account_set.id.into())
    }
}
