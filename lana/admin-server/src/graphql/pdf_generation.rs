use async_graphql::*;

use crate::primitives::*;

// Loan Agreement types
#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum LoanAgreementStatus {
    Pending,
    Completed,
    Failed,
}

impl From<lana_app::pdf_generation::LoanAgreementStatus> for LoanAgreementStatus {
    fn from(status: lana_app::pdf_generation::LoanAgreementStatus) -> Self {
        match status {
            lana_app::pdf_generation::LoanAgreementStatus::Pending => Self::Pending,
            lana_app::pdf_generation::LoanAgreementStatus::Completed => Self::Completed,
            lana_app::pdf_generation::LoanAgreementStatus::Failed => Self::Failed,
            lana_app::pdf_generation::LoanAgreementStatus::Removed => Self::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct LoanAgreement {
    id: ID,
    status: LoanAgreementStatus,
    created_at: Timestamp,
}

impl LoanAgreement {
    pub fn new(
        id: uuid::Uuid,
        status: LoanAgreementStatus,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id: id.to_string().into(),
            status,
            created_at: created_at.into(),
        }
    }
}

impl From<lana_app::pdf_generation::LoanAgreement> for LoanAgreement {
    fn from(domain_loan_agreement: lana_app::pdf_generation::LoanAgreement) -> Self {
        Self::new(
            domain_loan_agreement.id,
            domain_loan_agreement.status.into(),
            domain_loan_agreement.created_at,
        )
    }
}

// Credit Facility Export types
#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum CreditFacilityExportStatus {
    Pending,
    Completed,
    Failed,
}

impl From<lana_app::pdf_generation::CreditFacilityExportStatus> for CreditFacilityExportStatus {
    fn from(status: lana_app::pdf_generation::CreditFacilityExportStatus) -> Self {
        match status {
            lana_app::pdf_generation::CreditFacilityExportStatus::Pending => Self::Pending,
            lana_app::pdf_generation::CreditFacilityExportStatus::Completed => Self::Completed,
            lana_app::pdf_generation::CreditFacilityExportStatus::Failed => Self::Failed,
            lana_app::pdf_generation::CreditFacilityExportStatus::Removed => Self::Failed,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct CreditFacilityExport {
    id: ID,
    status: CreditFacilityExportStatus,
    created_at: Timestamp,
}

impl CreditFacilityExport {
    pub fn new(
        id: uuid::Uuid,
        status: CreditFacilityExportStatus,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id: id.to_string().into(),
            status,
            created_at: created_at.into(),
        }
    }
}

impl From<lana_app::pdf_generation::CreditFacilityExport> for CreditFacilityExport {
    fn from(domain_export: lana_app::pdf_generation::CreditFacilityExport) -> Self {
        Self::new(
            domain_export.id,
            domain_export.status.into(),
            domain_export.created_at,
        )
    }
}

// Unified PDF Generation types

// Enum representing different types of PDFs that can be generated
#[derive(async_graphql::Enum, Clone, Debug, PartialEq, Eq, Copy)]
pub enum PdfDocumentType {
    LoanAgreement,
    CreditFacilityExport,
}

// Union type representing the generated PDF document
#[derive(async_graphql::Union, Clone)]
pub enum PdfDocument {
    LoanAgreement(LoanAgreement),
    CreditFacilityExport(CreditFacilityExport),
}

// Input for generating a loan agreement
#[derive(InputObject)]
pub struct LoanAgreementInput {
    pub customer_id: UUID,
}

// Input for generating a credit facility export (no parameters needed)
#[derive(InputObject)]
pub struct CreditFacilityExportInput {
    // Placeholder field to make this a valid input object
    // Set to true to generate the export
    #[graphql(default = true)]
    pub generate: bool,
}

// Unified input for PDF generation using @oneOf
#[derive(OneofObject)]
pub enum PdfGenerateInput {
    #[graphql(name = "loanAgreement")]
    LoanAgreement(LoanAgreementInput),

    #[graphql(name = "creditFacilityExport")]
    CreditFacilityExport(CreditFacilityExportInput),
}

// Unified payload for PDF generation
#[derive(SimpleObject, Clone)]
pub struct PdfGeneratePayload {
    pub document: PdfDocument,
}

impl From<LoanAgreement> for PdfGeneratePayload {
    fn from(loan_agreement: LoanAgreement) -> Self {
        Self {
            document: PdfDocument::LoanAgreement(loan_agreement),
        }
    }
}

impl From<CreditFacilityExport> for PdfGeneratePayload {
    fn from(credit_facility_export: CreditFacilityExport) -> Self {
        Self {
            document: PdfDocument::CreditFacilityExport(credit_facility_export),
        }
    }
}

// Input for generating download links (unified)
#[derive(InputObject)]
pub struct PdfDownloadLinkGenerateInput {
    pub pdf_id: UUID,
}

// Unified payload for download link generation
#[derive(SimpleObject)]
pub struct PdfDownloadLinkGeneratePayload {
    pub pdf_id: UUID,
    pub link: String,
}

impl From<lana_app::document::GeneratedDocumentDownloadLink> for PdfDownloadLinkGeneratePayload {
    fn from(value: lana_app::document::GeneratedDocumentDownloadLink) -> Self {
        Self {
            pdf_id: UUID::from(value.document_id),
            link: value.link,
        }
    }
}
