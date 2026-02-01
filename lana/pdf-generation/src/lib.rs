use std::marker::PhantomData;

use ::job::{JobId, Jobs};
use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};
use core_customer::{
    CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject, Customers, kyc::CustomerKyc,
};
use document_storage::{
    Document, DocumentId, DocumentStatus, DocumentStorage, DocumentType,
    GeneratedDocumentDownloadLink, ReferenceId,
};
use obix::out::OutboxEventMarker;
use tracing::instrument;
use tracing_macros::record_error_severity;
use uuid::Uuid;

pub use core_credit::CreditFacilities;

mod document_types;
mod error;
mod jobs;
mod templates;

pub use document_types::*;
pub use error::*;
pub use jobs::*;
pub use primitives::{
    PERMISSION_SET_PDF_GENERATION, PdfGenerationId, PdfGenerationModuleAction,
    PdfGenerationModuleObject,
};
pub use templates::PdfTemplates;

pub mod primitives;

const LOAN_AGREEMENT_DOCUMENT_TYPE: DocumentType = DocumentType::new("loan_agreement");
const CREDIT_FACILITY_EXPORT_DOCUMENT_TYPE: DocumentType =
    DocumentType::new("credit_facility_export");

pub struct PdfGeneration<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    document_storage: DocumentStorage,
    generate_loan_agreement_job_spawner: GenerateLoanAgreementJobSpawner<Perms, E>,
    generate_credit_facility_export_job_spawner: GenerateCreditFacilityExportJobSpawner<Perms, E>,
    authz: Perms,
}

impl<Perms, E> Clone for PdfGeneration<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            document_storage: self.document_storage.clone(),
            generate_loan_agreement_job_spawner: self.generate_loan_agreement_job_spawner.clone(),
            generate_credit_facility_export_job_spawner: self
                .generate_credit_facility_export_job_spawner
                .clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms, E> PdfGeneration<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<governance::GovernanceEvent>
        + OutboxEventMarker<core_custody::CoreCustodyEvent>
        + OutboxEventMarker<core_price::CorePriceEvent>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<PdfGenerationModuleAction>
        + From<CoreCustomerAction>
        + From<CoreCreditAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<PdfGenerationModuleObject>
        + From<CustomerObject>
        + From<CoreCreditObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
{
    pub fn new(
        gotenberg_config: gotenberg::GotenbergConfig,
        customers: &Customers<Perms, E>,
        customer_kyc: &CustomerKyc<Perms, E>,
        credit_facilities: &CreditFacilities<Perms, E>,
        document_storage: &DocumentStorage,
        jobs: &mut Jobs,
        authz: &Perms,
    ) -> Self {
        let renderer = rendering::Renderer::new(gotenberg_config);
        let pdf_templates = PdfTemplates::new();

        // Initialize the job system for loan agreement generation
        let generate_loan_agreement_job_spawner =
            jobs.add_initializer(GenerateLoanAgreementJobInitializer::new(
                customers,
                customer_kyc,
                document_storage,
                pdf_templates.clone(),
                renderer.clone(),
            ));

        // Initialize the job system for credit facility export generation
        let generate_credit_facility_export_job_spawner =
            jobs.add_initializer(GenerateCreditFacilityExportJobInitializer::new(
                credit_facilities,
                customers,
                document_storage,
                pdf_templates.clone(),
                renderer.clone(),
                authz,
            ));

        Self {
            document_storage: document_storage.clone(),
            generate_loan_agreement_job_spawner,
            generate_credit_facility_export_job_spawner,
            authz: authz.clone(),
        }
    }

    #[record_error_severity]
    #[instrument(name = "pdf_generation.initiate_loan_agreement_generation", skip(self))]
    pub async fn initiate_loan_agreement_generation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<LoanAgreement, PdfGenerationError> {
        let customer_id = customer_id.into();

        self.authz
            .enforce_permission(
                sub,
                PdfGenerationModuleObject::all_pdfs(),
                PdfGenerationModuleAction::PDF_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let filename = format!("loan_agreement_{customer_id}.pdf");

        let mut db = self.document_storage.begin_op().await?;
        let document = self
            .document_storage
            .create_in_op(
                filename,
                "application/pdf",
                ReferenceId::from(customer_id),
                LOAN_AGREEMENT_DOCUMENT_TYPE,
                &mut db,
            )
            .await?;

        self.generate_loan_agreement_job_spawner
            .spawn_in_op(
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

    #[record_error_severity]
    #[instrument(
        name = "pdf_generation.initiate_credit_facility_export_generation",
        skip(self)
    )]
    pub async fn initiate_credit_facility_export_generation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<CreditFacilityExport, PdfGenerationError> {
        self.authz
            .enforce_permission(
                sub,
                PdfGenerationModuleObject::all_pdfs(),
                PdfGenerationModuleAction::PDF_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("credit_facilities_export_{timestamp}.pdf");

        let mut db = self.document_storage.begin_op().await?;
        let document = self
            .document_storage
            .create_in_op(
                filename,
                "application/pdf",
                ReferenceId::from(uuid::Uuid::new_v4()),
                CREDIT_FACILITY_EXPORT_DOCUMENT_TYPE,
                &mut db,
            )
            .await?;

        self.generate_credit_facility_export_job_spawner
            .spawn_in_op(
                &mut db,
                JobId::from(uuid::Uuid::from(document.id)),
                GenerateCreditFacilityExportConfig::<Perms, E> {
                    phantom: PhantomData,
                },
            )
            .await?;

        db.commit().await?;
        Ok(CreditFacilityExport::from(document))
    }

    #[record_error_severity]
    #[instrument(name = "pdf_generation.find_loan_agreement_by_id", skip(self))]
    pub async fn find_loan_agreement_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        pdf_id: impl Into<PdfGenerationId> + std::fmt::Debug,
    ) -> Result<Option<LoanAgreement>, PdfGenerationError> {
        let pdf_id = pdf_id.into();
        let document_id = DocumentId::from(pdf_id);

        self.authz
            .enforce_permission(
                sub,
                PdfGenerationModuleObject::all_pdfs(),
                PdfGenerationModuleAction::PDF_FIND,
            )
            .await?;

        match self.document_storage.find_by_id(document_id).await {
            Ok(document) => Ok(Some(LoanAgreement::from(document))),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "pdf_generation.find_credit_facility_export_by_id", skip(self))]
    pub async fn find_credit_facility_export_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        pdf_id: impl Into<PdfGenerationId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacilityExport>, PdfGenerationError> {
        let pdf_id = pdf_id.into();
        let document_id = DocumentId::from(pdf_id);

        self.authz
            .enforce_permission(
                sub,
                PdfGenerationModuleObject::all_pdfs(),
                PdfGenerationModuleAction::PDF_FIND,
            )
            .await?;

        match self.document_storage.find_by_id(document_id).await {
            Ok(document) => Ok(Some(CreditFacilityExport::from(document))),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "pdf_generation.generate_document_download_link", skip(self))]
    pub async fn generate_document_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        pdf_id: impl Into<PdfGenerationId> + std::fmt::Debug,
    ) -> Result<GeneratedDocumentDownloadLink, PdfGenerationError> {
        let pdf_id = pdf_id.into();
        self.authz
            .enforce_permission(
                sub,
                PdfGenerationModuleObject::all_pdfs(),
                PdfGenerationModuleAction::PDF_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let link = self.document_storage.generate_download_link(pdf_id).await?;

        Ok(link)
    }
}

// Simple loan agreement types (not using the full entity system)
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

// Credit facility export types
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreditFacilityExportStatus {
    Pending,
    Completed,
    Failed,
    Removed,
}

#[derive(Clone, Debug)]
pub struct CreditFacilityExport {
    pub id: Uuid,
    pub status: CreditFacilityExportStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Document> for CreditFacilityExport {
    fn from(document: Document) -> CreditFacilityExport {
        CreditFacilityExport {
            id: document.id.into(),
            status: document.status.into(),
            created_at: document.created_at(),
        }
    }
}

impl From<DocumentStatus> for CreditFacilityExportStatus {
    fn from(document_status: DocumentStatus) -> CreditFacilityExportStatus {
        match document_status {
            DocumentStatus::Active => CreditFacilityExportStatus::Completed,
            DocumentStatus::Archived => CreditFacilityExportStatus::Removed,
            DocumentStatus::Deleted => CreditFacilityExportStatus::Removed,
            DocumentStatus::Failed => CreditFacilityExportStatus::Failed,
            DocumentStatus::New => CreditFacilityExportStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gotenberg_config() -> gotenberg::GotenbergConfig {
        gotenberg::GotenbergConfig::default()
    }

    #[test]
    fn test_pdf_generation_config() -> Result<(), error::PdfGenerationError> {
        // Test that the renderer can be created with config
        let _renderer = rendering::Renderer::new(test_gotenberg_config());

        // Test embedded templates
        let pdf_templates = PdfTemplates::new();
        let data = serde_json::json!({
            "full_name": "Test User",
            "email": "test@example.com",
            "customer_id": "test-123",
            "telegram_id": "test_telegram",
            "date": "2025-01-01"
        });

        let result = pdf_templates.render_template("loan_agreement", &data)?;
        assert!(result.contains("Test User"));
        assert!(result.contains("test@example.com"));

        Ok(())
    }
}
