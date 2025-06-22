mod entity;
pub mod error;
mod generate;
mod job;
mod primitives;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;



use es_entity::ListDirection;

use crate::{CoreDocumentStorageAction, CoreDocumentStorageObject};
use super::primitives::{LoanAgreementId, CustomerId};

use job::Jobs;
use cloud_storage::Storage;

#[cfg(feature = "json-schema")]
pub use entity::LoanAgreementEvent;
pub use entity::{LoanAgreement, NewLoanAgreement};
use error::*;
use job::*;
pub use primitives::*;
pub use repo::loan_agreement_cursor::LoanAgreementsByCreatedAtCursor;
use repo::*;

#[derive(Clone)]
pub struct LoanAgreements<Perms>
where
    Perms: PermissionCheck,
{
    repo: LoanAgreementRepo,
    authz: Perms,
    jobs: Jobs,
    storage: Storage,
}

impl<Perms> LoanAgreements<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &Jobs,
        storage: &Storage,
    ) -> Self {
        let repo = LoanAgreementRepo::new(pool);

        jobs.add_initializer(GenerateLoanAgreementInitializer::new(
            &repo,
            storage,
            authz.audit(),
        ));

        Self {
            repo,
            authz: authz.clone(),
            jobs: jobs.clone(),
            storage: storage.clone(),
        }
    }

    #[instrument(name = "core_document_storage.loan_agreement.create", skip(self), err)]
    pub async fn create_loan_agreement(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<LoanAgreement, LoanAgreementError> {
        let customer_id = customer_id.into();
        let id = LoanAgreementId::new();

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDocumentStorageObject::all_loan_agreements(),
                CoreDocumentStorageAction::LOAN_AGREEMENT_CREATE,
            )
            .await?;

        let new_agreement = NewLoanAgreement::builder()
            .id(id)
            .customer_id(customer_id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new Loan Agreement");

        let mut db = self.repo.begin_op().await?;
        let agreement = self.repo.create_in_op(&mut db, new_agreement).await?;
        self.jobs
            .create_and_spawn_in_op::<GenerateLoanAgreementConfig<Perms>>(
                &mut db,
                agreement.id,
                GenerateLoanAgreementConfig {
                    loan_agreement_id: agreement.id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        db.commit().await?;
        Ok(agreement)
    }

    #[instrument(name = "core_document_storage.loan_agreement.generate_download_link", skip(self), err)]
    pub async fn generate_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        loan_agreement_id: impl Into<LoanAgreementId> + std::fmt::Debug,
    ) -> Result<GeneratedLoanAgreementDownloadLink, LoanAgreementError> {
        let loan_agreement_id = loan_agreement_id.into();

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDocumentStorageObject::loan_agreement(loan_agreement_id),
                CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let agreement = self.repo.find_by_id(loan_agreement_id).await?;

        if agreement.status != LoanAgreementStatus::Completed {
            return Err(LoanAgreementError::LoanAgreementNotReady);
        }

        let link = self
            .storage
            .generate_download_link(cloud_storage::LocationInStorage {
                path_in_storage: agreement.storage_path.ok_or(LoanAgreementError::LoanAgreementFileNotFound)?,
            })
            .await?;

        Ok(GeneratedLoanAgreementDownloadLink {
            loan_agreement_id,
            link,
        })
    }

    #[instrument(name = "core_document_storage.loan_agreement.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        loan_agreement_id: impl Into<LoanAgreementId> + std::fmt::Debug,
    ) -> Result<Option<LoanAgreement>, LoanAgreementError> {
        match self.repo.find_by_id(loan_agreement_id.into()).await {
            Ok(agreement) => Ok(Some(agreement)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}