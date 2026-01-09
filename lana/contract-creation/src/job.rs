use std::marker::PhantomData;

use async_trait::async_trait;
use authz::PermissionCheck;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject};
use document_storage::{DocumentId, DocumentStorage};
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use super::{LoanAgreementData, templates::ContractTemplates};
use crate::{Applicants, Customers};

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

impl<Perms, E> Clone for GenerateLoanAgreementConfig<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            customer_id: self.customer_id,
            phantom: PhantomData,
        }
    }
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
    type Config = GenerateLoanAgreementConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        GENERATE_LOAN_AGREEMENT_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
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
    #[tracing::instrument(
        name = "contract_creation.generate_loan_agreement_job.run",
        skip_all,
        fields(
            job_id = %current_job.id(),
            job_attempt = current_job.attempt(),
            customer_id = %self.config.customer_id
        ),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // Find the customer for this loan agreement
        let customer = self
            .customers
            .find_by_id_without_audit(self.config.customer_id)
            .await?;

        // Get applicant information from Sumsub if available
        let (full_name, address, country) = if customer.applicant_id.is_some() {
            match self
                .applicants
                .get_applicant_info_without_audit(self.config.customer_id)
                .await
            {
                Ok(applicant_info) => (
                    applicant_info
                        .full_name()
                        .unwrap_or_else(|| "N/A".to_string()),
                    applicant_info.primary_address().map(|s| s.to_string()),
                    applicant_info.nationality().map(|s| s.to_string()),
                ),
                Err(_) => ("N/A (applicant info not available)".to_string(), None, None),
            }
        } else {
            ("N/A (customer has no applicant)".to_string(), None, None)
        };

        let loan_data = LoanAgreementData::new(
            customer.email.clone(),
            customer.telegram_id.clone(),
            self.config.customer_id,
            full_name,
            address,
            country,
        );

        let content = self
            .contract_templates
            .render_template("loan_agreement", &loan_data)?;
        let pdf_bytes = self.renderer.render_template_to_pdf(&content)?;

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

pub type GenerateLoanAgreementJobSpawner<Perms, E> =
    JobSpawner<GenerateLoanAgreementConfig<Perms, E>>;
