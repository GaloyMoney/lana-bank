mod entity;
pub mod error;
mod generate;
mod job;
mod repo;

use tracing::instrument;

use super::CoreAccountingAction;
use super::CoreAccountingObject;
use super::audit::AuditSvc;
use super::ledger_account::LedgerAccounts;
use super::primitives::{AccountingCsvId, LedgerAccountId, Subject};
use super::storage::Storage;
use authz::PermissionCheck;

pub use entity::*;
use error::*;
use generate::*;
use job::*;
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
        jobs.add_initializer(GenerateAccountingCsvInitializer::new(
            &repo,
            storage,
            ledger_accounts,
        ));

        Self {
            repo,
            authz: authz.clone(),
            jobs: jobs.clone(),
            storage: storage.clone(),
        }
    }

    pub async fn create_ledger_account_csv(
        &self,
        sub: &Subject,
        ledger_account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<AccountingCsv, AccountingCsvError> {
        let ledger_account_id = ledger_account_id.into();

        let csv_type = AccountingCsvType::LedgerAccount { ledger_account_id };
        let new_csv = NewAccountingCsv::builder()
            .id(AccountingCsvId::new())
            .csv_type(csv_type)
            .build()
            .expect("Could not build new Accounting CSV");

        let mut db = self.repo.begin_op().await?;
        let csv = self.repo.create_in_op(&mut db, new_csv).await?;

        self.jobs
            .create_and_spawn_in_op(
                &mut db,
                csv.id,
                GenerateAccountingCsvConfig {
                    accounting_csv_id: csv.id,
                },
            )
            .await?;

        db.commit().await?;
        Ok(csv)
    }

    pub async fn generate_download_link(
        &self,
        sub: &Subject,
        id: impl Into<AccountingCsvId> + std::fmt::Debug,
    ) -> Result<GeneratedAccountingCsvDownloadLink, AccountingCsvError> {
        unimplemented!()
    }
}
