use std::fs;
use std::path::PathBuf;

use crate::contract_creation::error::ContractCreationError;
use crate::customer::Customers;
use crate::{authorization::Authorization, contract_creation::config::ContractCreationConfig};
use authz::PermissionCheck;
use document_storage::{DocumentStorage, DocumentType, ReferenceId};
use job::{JobId, Jobs};
use rbac_types::{AppAction, AppObject, LanaAction, LanaObject, Subject};

#[derive(Clone)]
pub struct ContractCreation {
    renderer: rendering::Renderer,
    template_dir: PathBuf,
    document_storage: DocumentStorage,
    jobs: Jobs,
    authz: Authorization,
}

impl ContractCreation {
    pub async fn init(
        config: ContractCreationConfig,
        customers: &Customers,
        document_storage: &DocumentStorage,
        jobs: &Jobs,
        authz: &Authorization,
    ) -> Result<Self, ContractCreationError> {
        let renderer = rendering::Renderer::new(config.pdf_config_file).await?;

        // Initialize the job system for contract creation
        jobs.add_initializer(super::GenerateLoanAgreementJobInitializer::new(
            customers,
            document_storage,
            config.template_dir.clone(),
            renderer.clone(),
        ));

        Ok(Self {
            renderer,
            template_dir: config.template_dir,
            document_storage: document_storage.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
        })
    }

    async fn load_template(&self, template_name: &str) -> Result<String, ContractCreationError> {
        let template_path = self.template_dir.join(format!("{}.md.hbs", template_name));

        if !template_path.exists() {
            return Err(ContractCreationError::TemplateNotFound(
                template_name.to_string(),
            ));
        }

        let template_content = fs::read_to_string(&template_path)?;
        Ok(template_content)
    }

    pub async fn generate_contract_pdf_from_template<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<Vec<u8>, ContractCreationError> {
        let template_content = self.load_template(template_name).await?;
        let pdf_bytes = self
            .renderer
            .render_template_to_pdf(&template_content, data)
            .await?;
        Ok(pdf_bytes)
    }

    pub async fn generate_loan_agreement(
        &self,
        sub: &Subject,
        customer_id: crate::customer::CustomerId,
    ) -> Result<SimpleLoanAgreement, ContractCreationError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                LanaObject::App(AppObject::all_contract_creation()),
                LanaAction::App(AppAction::ContractCreation(
                    rbac_types::ContractCreationAction::Create,
                )),
            )
            .await?;

        let id = uuid::Uuid::new_v4();
        let created_at = chrono::Utc::now();

        let loan_agreement = SimpleLoanAgreement {
            id,
            customer_id,
            status: SimpleLoanAgreementStatus::Pending,
            storage_path: None,
            created_at,
        };

        // Create a filename for the PDF
        let filename = format!("loan_agreement_{:?}.pdf", customer_id);

        // Define document type for loan agreements
        const LOAN_AGREEMENT_DOCUMENT_TYPE: DocumentType = DocumentType::new("loan_agreement");

        let mut db = self.document_storage.begin_op().await?;
        let document = self
            .document_storage
            .create_in_op(
                audit_info.clone(),
                filename,
                "application/pdf",
                ReferenceId::from(customer_id),
                LOAN_AGREEMENT_DOCUMENT_TYPE,
                &mut db,
            )
            .await?;

        // Create and spawn the job using the job library, using document ID as job ID
        self.jobs
            .create_and_spawn_in_op::<super::GenerateLoanAgreementConfig>(
                &mut db,
                JobId::from(uuid::Uuid::from(document.id)),
                super::GenerateLoanAgreementConfig { customer_id },
            )
            .await?;

        db.commit().await?;

        // should I return document instead?
        Ok(loan_agreement)
    }
}

/// Data structure for loan agreement template
#[derive(serde::Serialize)]
pub struct LoanAgreementData {
    pub email: String,
}

impl LoanAgreementData {
    pub fn new(email: String) -> Self {
        Self { email }
    }
}

// Simple loan agreement types for now (not using the full entity system)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimpleLoanAgreementStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct SimpleLoanAgreement {
    pub id: uuid::Uuid,
    pub customer_id: crate::customer::CustomerId,
    pub status: SimpleLoanAgreementStatus,
    pub storage_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
