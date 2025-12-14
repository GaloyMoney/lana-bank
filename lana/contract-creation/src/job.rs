use std::marker::PhantomData;

use async_trait::async_trait;
use authz::PermissionCheck;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject};
use document_storage::{DocumentId, DocumentStorage};
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use outbox::OutboxEventMarker;
use tracing_macros::record_error_severity;

use super::templates::ContractTemplates;
use crate::{Applicants, Customers, generate_loan_agreement_pdf};

#[derive(Serialize, Deserialize)]
pub struct GenerateLoanAgreementConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub customer_id: CustomerId,
    #[serde(skip)]
    pub phantom: PhantomData<(Perms, E)>,
}

impl<Perms, E> JobConfig for GenerateLoanAgreementConfig<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + Send + Sync,
{
    type Initializer = GenerateLoanAgreementJobInitializer<Perms, E>;
}

pub struct GenerateLoanAgreementJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    customers: Customers<Perms, E>,
    applicants: Applicants<Perms, E>,
    document_storage: DocumentStorage,
    contract_templates: ContractTemplates,
    renderer: rendering::Renderer,
}

impl<Perms, E> GenerateLoanAgreementJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        customers: &Customers<Perms, E>,
        applicants: &Applicants<Perms, E>,
        document_storage: &DocumentStorage,
        contract_templates: ContractTemplates,
        renderer: rendering::Renderer,
    ) -> Self {
        Self {
            customers: customers.clone(),
            applicants: applicants.clone(),
            document_storage: document_storage.clone(),
            contract_templates,
            renderer,
        }
    }
}

pub const GENERATE_LOAN_AGREEMENT_JOB: JobType = JobType::new("task.generate-loan-agreement");

impl<Perms, E> JobInitializer for GenerateLoanAgreementJobInitializer<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + Send + Sync,
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
            customers: self.customers.clone(),
            applicants: self.applicants.clone(),
            document_storage: self.document_storage.clone(),
            contract_templates: self.contract_templates.clone(),
            renderer: self.renderer.clone(),
        }))
    }
}

pub struct GenerateLoanAgreementJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    config: GenerateLoanAgreementConfig<Perms, E>,
    customers: Customers<Perms, E>,
    applicants: Applicants<Perms, E>,
    document_storage: DocumentStorage,
    contract_templates: ContractTemplates,
    renderer: rendering::Renderer,
}

#[async_trait]
impl<Perms, E> JobRunner for GenerateLoanAgreementJobRunner<Perms, E>
where
    Perms: PermissionCheck + Send + Sync,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + Send + Sync,
{
    #[record_error_severity]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Generate the PDF using the reusable function from the use case layer
        let pdf_bytes = generate_loan_agreement_pdf(
            self.config.customer_id,
            &self.customers,
            &self.applicants,
            &self.contract_templates,
            &self.renderer,
        )
        .await?;

        // Convert job ID to document ID (they should be the same as per the pattern)
        let document_id = DocumentId::from(uuid::Uuid::from(*current_job.id()));

        // Find the document that was created for this job
        let mut document = self.document_storage.find_by_id(document_id).await?;

        // Upload the PDF content to the document
        self.document_storage
            .upload(pdf_bytes, &mut document)
            .await?;

        Ok(JobCompletion::Complete)
    }
}
