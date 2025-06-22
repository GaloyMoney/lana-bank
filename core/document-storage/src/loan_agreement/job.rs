use async_trait::async_trait;

use authz::PermissionCheck;
use audit::AuditSvc;
use cloud_storage::Storage;
use job::*;
use serde::{Deserialize, Serialize};

use crate::{CoreDocumentStorageAction, CoreDocumentStorageObject};
use super::{
    error::LoanAgreementError, 
    generate::GenerateLoanAgreementPdf,
    primitives::LoanAgreementId, 
    repo::LoanAgreementRepo,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateLoanAgreementConfig<Perms> {
    pub loan_agreement_id: LoanAgreementId,
    pub _phantom: std::marker::PhantomData<Perms>,
}

impl<Perms> JobConfig for GenerateLoanAgreementConfig<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    type Initializer = GenerateLoanAgreementInitializer<Perms>;
}

pub struct GenerateLoanAgreementInitializer<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    repo: LoanAgreementRepo,
    storage: Storage,
    audit: Perms::Audit,
}

impl<Perms> GenerateLoanAgreementInitializer<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    pub fn new(
        repo: &LoanAgreementRepo,
        storage: &Storage,
        audit: &Perms::Audit,
    ) -> Self {
        Self {
            repo: repo.clone(),
            storage: storage.clone(),
            audit: audit.clone(),
        }
    }
}

pub const GENERATE_LOAN_AGREEMENT_JOB: JobType = JobType::new("generate-loan-agreement");

impl<Perms> JobInitializer for GenerateLoanAgreementInitializer<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        GENERATE_LOAN_AGREEMENT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GenerateLoanAgreementJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            storage: self.storage.clone(),
            generator: GenerateLoanAgreementPdf::new(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct GenerateLoanAgreementJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    config: GenerateLoanAgreementConfig<Perms>,
    repo: LoanAgreementRepo,
    storage: Storage,
    generator: GenerateLoanAgreementPdf,
    audit: Perms::Audit,
}

#[async_trait]
impl<Perms> JobRunner for GenerateLoanAgreementJobRunner<Perms>
where
    Perms: authz::PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDocumentStorageObject>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut agreement = self.repo.find_by_id(self.config.loan_agreement_id).await?;
        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db.tx(),
                CoreDocumentStorageObject::all_loan_agreements(),
                CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE,
            )
            .await?;

        let generation_result = self
            .generator
            .generate_pdf(agreement.customer_id)
            .await;

        match generation_result {
            Ok((pdf_data, filename)) => {
                let path_in_bucket = format!("loan_agreements/{}.pdf", agreement.id);
                match self
                    .storage
                    .upload(pdf_data, &path_in_bucket, "application/pdf")
                    .await
                {
                    Ok(_) => {
                        let _ = agreement.file_generated(path_in_bucket, filename, audit_info);
                    }
                    Err(e) => {
                        let _ = agreement.generation_failed(e.to_string(), audit_info);
                    }
                }
            }
            Err(e) => {
                let _ = agreement.generation_failed(e.to_string(), audit_info);
            }
        }

        self.repo.update_in_op(&mut db, &mut agreement).await?;
        let (now, tx) = (db.now(), db.into_tx());
        let db_static = es_entity::DbOp::new(tx, now);
        Ok(JobCompletion::CompleteWithOp(db_static))
    }
}