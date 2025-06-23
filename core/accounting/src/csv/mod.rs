mod entity;
pub mod error;
mod job;
mod primitives;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::Jobs;
use crate::Storage;

use es_entity::ListDirection;

use super::{
    CoreAccountingAction, CoreAccountingObject,
    ledger_account::LedgerAccounts,
    primitives::{AccountingCsvId, LedgerAccountId},
};

#[cfg(feature = "json-schema")]
pub use entity::AccountingCsvEvent;
pub use entity::{AccountingCsv, NewAccountingCsv};
use error::*;
use job::*;
pub use primitives::*;
pub use repo::accounting_csv_cursor::AccountingCsvsByCreatedAtCursor;
use repo::*;

#[derive(Clone)]
pub struct AccountingCsvs<Perms>
where
    Perms: PermissionCheck,
{
    repo: AccountingCsvRepo,
    authz: Perms,
    jobs: Jobs,
    storage: Storage,
}

impl<Perms> AccountingCsvs<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &Jobs,
        storage: &Storage,
        ledger_accounts: &LedgerAccounts<Perms>,
    ) -> Self {
        let repo = AccountingCsvRepo::new(pool);

        jobs.add_initializer(GenerateAccountingCsvInit::new(
            &repo,
            storage,
            ledger_accounts,
            authz.audit(),
        ));

        Self {
            repo,
            authz: authz.clone(),
            jobs: jobs.clone(),
            storage: storage.clone(),
        }
    }

    #[instrument(name = "core_accounting.csv.create", skip(self), err)]
    pub async fn create_ledger_account_csv(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<AccountingCsv, AccountingCsvError> {
        let ledger_account_id = ledger_account_id.into();
        let id = AccountingCsvId::new();

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_CREATE,
            )
            .await?;

        let new_csv = NewAccountingCsv::builder()
            .id(id)
            .csv_type(AccountingCsvType::LedgerAccount)
            .ledger_account_id(ledger_account_id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new Accounting CSV");

        let mut db = self.repo.begin_op().await?;
        let csv = self.repo.create_in_op(&mut db, new_csv).await?;
        self.jobs
            .create_and_spawn_in_op::<GenerateAccountingCsvConfig<Perms>>(
                &mut db,
                csv.id,
                GenerateAccountingCsvConfig {
                    accounting_csv_id: csv.id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        db.commit().await?;
        Ok(csv)
    }

    #[instrument(name = "core_accounting.csv.generate_download_link", skip(self), err)]
    pub async fn generate_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        accounting_csv_id: AccountingCsvId,
    ) -> Result<GeneratedAccountingCsvDownloadLink, AccountingCsvError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let mut csv = self.repo.find_by_id(accounting_csv_id).await?;
        let location = csv.download_link_generated(audit_info)?;

        let url = self.storage.generate_download_link(location).await?;
        self.repo.update(&mut csv).await?;

        Ok(GeneratedAccountingCsvDownloadLink {
            accounting_csv_id,
            link: AccountingCsvDownloadLink {
                csv_type: csv.csv_type,
                url,
            },
        })
    }

    #[instrument(
        name = "core_accounting.csv.list_for_ledger_account_id",
        skip(self),
        err
    )]
    pub async fn list_for_ledger_account_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<AccountingCsvsByCreatedAtCursor>,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<AccountingCsv, AccountingCsvsByCreatedAtCursor>,
        AccountingCsvError,
    > {
        let ledger_account_id = ledger_account_id.into();

        let _audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_csvs(),
                CoreAccountingAction::ACCOUNTING_CSV_LIST,
            )
            .await?;

        let csvs = self
            .repo
            .list_for_ledger_account_id_by_created_at(
                Some(ledger_account_id),
                query,
                ListDirection::Descending,
            )
            .await?;

        Ok(csvs)
    }

    #[instrument(name = "core_accounting.csv.find_all", skip(self), err)]
    pub async fn find_all<T: From<AccountingCsv>>(
        &self,
        ids: &[AccountingCsvId],
    ) -> Result<std::collections::HashMap<AccountingCsvId, T>, AccountingCsvError> {
        self.repo.find_all(ids).await
    }
}
