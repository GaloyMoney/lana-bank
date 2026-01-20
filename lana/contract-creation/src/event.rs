use document_storage::DocumentId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum ContractCreationEvent {
    LoanAgreementGenerated { loan_agreement_id: DocumentId },
}
