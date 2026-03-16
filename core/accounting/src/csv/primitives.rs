use document_storage::DocumentId;
use serde::{Deserialize, Serialize};

es_entity::entity_id! {
    AccountingCsvDocumentId;
    AccountingCsvDocumentId => DocumentId
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, strum::Display, strum::EnumString, Copy,
)]
#[serde(rename_all = "snake_case")]
pub enum AccountingCsvType {
    LedgerAccount,
    ProfitAndLoss,
    BalanceSheet,
}

#[derive(Debug, Clone)]
pub struct AccountingCsvDownloadLink {
    pub csv_type: AccountingCsvType,
    pub url: String,
}
