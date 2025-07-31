use std::marker::PhantomData;

use ::job::{JobId, Jobs};
use audit::AuditSvc;
use authz::PermissionCheck;
use core_applicant::Applicants;
use core_customer::{CoreCustomerAction, CustomerId, CustomerObject, Customers};
use document_storage::{
    Document, DocumentId, DocumentStatus, DocumentStorage, DocumentType,
    GeneratedDocumentDownloadLink, ReferenceId,
};
use lana_events::{CoreCustomerEvent, OutboxEventMarker};
use lana_ids::ContractCreationId;
use uuid::Uuid;

mod error;
pub mod job;
mod templates;

pub use error::*;
pub use job::*;
pub use primitives::{ContractModuleAction, ContractModuleObject};

use tracing::instrument;

pub mod primitives;
const LOAN_AGREEMENT_DOCUMENT_TYPE: DocumentType = DocumentType::new("loan_agreement");

pub struct ContractCreation<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    document_storage: DocumentStorage,
    jobs: Jobs,
    authz: Perms,
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms: PermissionCheck, E> Clone for ContractCreation<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            document_storage: self.document_storage.clone(),
            jobs: self.jobs.clone(),
            authz: self.authz.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Perms, E> ContractCreation<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<ContractModuleAction> + From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<ContractModuleObject> + From<CustomerObject>,
{
    pub fn try_new(
        customers: &Customers<Perms, E>,
        applicants: &Applicants<Perms, E>,
        document_storage: &DocumentStorage,
        jobs: &Jobs,
        authz: &Perms,
    ) -> Result<Self, ContractCreationError> {
        let renderer = rendering::Renderer::new();
        let contract_templates = templates::ContractTemplates::try_new()?;

        // Initialize the job system for contract creation
        jobs.add_initializer(GenerateLoanAgreementJobInitializer::new(
            customers,
            applicants,
            document_storage,
            contract_templates,
            renderer.clone(),
        ));

        Ok(Self {
            document_storage: document_storage.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            _phantom: std::marker::PhantomData,
        })
    }

    #[instrument(name = "contract.initiate_loan_agreement_generation", skip(self), err)]
    pub async fn initiate_loan_agreement_generation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<LoanAgreement, ContractCreationError> {
        let customer_id = customer_id.into();

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                ContractModuleObject::all_contracts(),
                ContractModuleAction::CONTRACT_CREATE,
                // LanaObject::App(AppObject::all_contract_creation()),
                // LanaAction::App(AppAction::ContractCreation(
                //     rbac_types::ContractCreationAction::Generate,
                // )),
            )
            .await?;

        let filename = format!("loan_agreement_{customer_id}.pdf");

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

        self.jobs
            .create_and_spawn_in_op::<GenerateLoanAgreementConfig<Perms, E>>(
                &mut db,
                JobId::from(uuid::Uuid::from(document.id)),
                GenerateLoanAgreementConfig::<Perms, E> {
                    customer_id,
                    phantom: PhantomData,
                },
            )
            .await?;

        db.commit().await?;
        Ok(LoanAgreement::from(document))
    }

    #[instrument(name = "contract.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        contract_id: impl Into<ContractCreationId> + std::fmt::Debug,
    ) -> Result<Option<LoanAgreement>, ContractCreationError> {
        let contract_id = contract_id.into();
        let document_id = DocumentId::from(contract_id);

        let _audit_info = self
            .authz
            .enforce_permission(
                sub,
                ContractModuleObject::all_contracts(),
                ContractModuleAction::CONTRACT_CREATE,
                // LanaObject::App(AppObject::all_contract_creation()),
                // LanaAction::App(AppAction::ContractCreation(
                //     rbac_types::ContractCreationAction::Find,
                // )),
            )
            .await?;

        match self.document_storage.find_by_id(document_id).await {
            Ok(document) => Ok(Some(LoanAgreement::from(document))),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "contract.generate_document_download_link", skip(self), err)]
    pub async fn generate_document_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        contract_id: impl Into<ContractCreationId> + std::fmt::Debug,
    ) -> Result<GeneratedDocumentDownloadLink, ContractCreationError> {
        let contract_id = contract_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                ContractModuleObject::all_contracts(),
                ContractModuleAction::CONTRACT_CREATE,
                // LanaObject::App(AppObject::all_contract_creation()),
                // LanaAction::App(AppAction::ContractCreation(
                //     rbac_types::ContractCreationAction::GenerateDownloadLink,
                // )),
            )
            .await?;

        let link = self
            .document_storage
            .generate_download_link(audit_info, contract_id)
            .await?;

        Ok(link)
    }
}

impl From<Document> for LoanAgreement {
    fn from(document: Document) -> LoanAgreement {
        LoanAgreement {
            id: document.id.into(),
            status: document.status.into(),
            created_at: document.created_at(),
        }
    }
}

impl From<DocumentStatus> for LoanAgreementStatus {
    fn from(document_status: DocumentStatus) -> LoanAgreementStatus {
        match document_status {
            DocumentStatus::Active => LoanAgreementStatus::Completed,
            DocumentStatus::Archived => LoanAgreementStatus::Removed,
            DocumentStatus::Deleted => LoanAgreementStatus::Removed,
            DocumentStatus::Failed => LoanAgreementStatus::Failed,
            DocumentStatus::New => LoanAgreementStatus::Pending,
        }
    }
}

/// Data structure for loan agreement template
#[derive(serde::Serialize)]
pub struct LoanAgreementData {
    pub email: String,
    pub full_name: String,
    pub address: Option<String>,
    pub country: Option<String>,
    pub customer_id: String,
    pub telegram_id: String,
    pub date: String,
}

impl LoanAgreementData {
    pub fn new(
        email: String,
        telegram_id: String,
        customer_id: CustomerId,
        full_name: String,
        address: Option<String>,
        country: Option<String>,
    ) -> Self {
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        Self {
            email,
            full_name,
            address,
            country,
            customer_id: customer_id.to_string(),
            telegram_id,
            date,
        }
    }
}

// Simple loan agreement types for now (not using the full entity system)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoanAgreementStatus {
    Pending,
    Completed,
    Failed,
    Removed,
}

#[derive(Clone, Debug)]
pub struct LoanAgreement {
    pub id: Uuid,
    pub status: LoanAgreementStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_contract_creation_config() -> Result<(), error::ContractCreationError> {
        // Test that the embedded PDF config works correctly
        // Verify that renderer can be created with embedded config
        let _renderer = rendering::Renderer::new();

        // Test embedded templates
        let contract_templates = templates::ContractTemplates::try_new()?;
        let data = serde_json::json!({
            "full_name": "Test User",
            "email": "test@example.com",
            "customer_id": "test-123",
            "telegram_id": "test_telegram",
            "date": "2025-01-01"
        });

        let result = contract_templates.render_template("loan_agreement", &data)?;
        assert!(result.contains("Test User"));
        assert!(result.contains("test@example.com"));

        Ok(())
    }
}
