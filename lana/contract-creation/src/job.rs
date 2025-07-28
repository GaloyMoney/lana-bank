use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use core_customer::CustomerId;
use document_storage::{DocumentId, DocumentStorage};
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};

use super::{LoanAgreementData, error::ContractCreationError, templates::ContractTemplates};

// Type aliases that match the main app's concrete types
type LanaAudit = audit::Audit<rbac_types::Subject, rbac_types::LanaObject, rbac_types::LanaAction>;
type Authorization = authz::Authorization<LanaAudit, core_access::AuthRoleToken>;
type Customers = core_customer::Customers<Authorization, lana_events::LanaEvent>;
type Applicants = core_applicant::Applicants<Authorization, lana_events::LanaEvent>;

#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateLoanAgreementConfig {
    pub customer_id: CustomerId,
}

impl JobConfig for GenerateLoanAgreementConfig {
    type Initializer = GenerateLoanAgreementJobInitializer;
}

pub struct GenerateLoanAgreementJobInitializer {
    customers: Customers,
    applicants: Applicants,
    document_storage: DocumentStorage,
    contract_templates: ContractTemplates,
    renderer: rendering::Renderer,
}

impl GenerateLoanAgreementJobInitializer {
    pub fn new(
        customers: &Customers,
        applicants: &Applicants,
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

pub const GENERATE_LOAN_AGREEMENT_JOB: JobType = JobType::new("generate-loan-agreement");

impl JobInitializer for GenerateLoanAgreementJobInitializer {
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

pub struct GenerateLoanAgreementJobRunner {
    config: GenerateLoanAgreementConfig,
    customers: Customers,
    applicants: Applicants,
    document_storage: DocumentStorage,
    contract_templates: ContractTemplates,
    renderer: rendering::Renderer,
}

impl GenerateLoanAgreementJobRunner {
    #[tracing::instrument(
        name = "lana.contract_creation.generate_contract_pdf_from_template",
        skip(self, data),
        err
    )]
    async fn generate_contract_pdf_from_template<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<Vec<u8>, ContractCreationError> {
        let template_content = self
            .contract_templates
            .render_template(template_name, data)?;
        let pdf_bytes = self
            .renderer
            .render_template_to_pdf(&template_content, data)?;
        Ok(pdf_bytes)
    }
}

#[async_trait]
impl JobRunner for GenerateLoanAgreementJobRunner {
    #[tracing::instrument(
        name = "lana.contract_creation.generate_loan_agreement",
        skip_all,
        fields(
            job_id = %current_job.id(),
            job_attempt = current_job.attempt()
        ),
        err
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

        // Generate the PDF bytes
        let pdf_bytes = self
            .generate_contract_pdf_from_template("loan_agreement", &loan_data)
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
