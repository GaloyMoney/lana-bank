use thiserror::Error;
use tracing::Level;
use tracing_utils::ErrorSeverity;

#[derive(Error, Debug)]
pub enum PdfGenerationError {
    #[error("PdfGenerationError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PdfGenerationError - EsEntity: {0}")]
    EsEntity(#[from] es_entity::EsEntityError),
    #[error("PdfGenerationError - DocumentStorage: {0}")]
    DocumentStorage(#[from] document_storage::error::DocumentStorageError),
    #[error("PdfGenerationError - Customer: {0}")]
    Customer(#[from] core_customer::error::CustomerError),
    #[error("PdfGenerationError - CustomerKyc: {0}")]
    CustomerKyc(#[from] core_customer::kyc::error::KycError),
    #[error("PdfGenerationError - Credit: {0}")]
    Credit(#[from] core_credit::error::CoreCreditError),
    #[error("PdfGenerationError - Authz: {0}")]
    Authz(#[from] authz::error::AuthorizationError),
    #[error("PdfGenerationError - Rendering: {0}")]
    Rendering(#[from] rendering::RenderingError),
    #[error("PdfGenerationError - Job: {0}")]
    Job(#[from] job::error::JobError),
    #[error("PdfGenerationError - UnknownDocumentType: {0}")]
    UnknownDocumentType(String),
}

impl ErrorSeverity for PdfGenerationError {
    fn severity(&self) -> Level {
        match self {
            Self::Sqlx(_) => Level::ERROR,
            Self::EsEntity(_) => Level::ERROR,
            Self::DocumentStorage(e) => e.severity(),
            Self::Customer(e) => e.severity(),
            Self::CustomerKyc(e) => e.severity(),
            Self::Credit(e) => e.severity(),
            Self::Authz(e) => e.severity(),
            Self::Rendering(_) => Level::ERROR,
            Self::Job(_) => Level::ERROR,
            Self::UnknownDocumentType(_) => Level::ERROR,
        }
    }
}

impl PdfGenerationError {
    pub fn was_not_found(&self) -> bool {
        matches!(self, Self::DocumentStorage(e) if e.was_not_found())
    }
}
