use serde::{Deserialize, Serialize};

use crate::primitives::AccountingCsvId;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, strum::Display, strum::EnumString, Copy,
)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AccountingCsvType {
    LedgerAccount,
    ProfitAndLoss,
    BalanceSheet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum AccountingCsvStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct AccountingCsvDownloadLink {
    pub csv_type: AccountingCsvType,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedAccountingCsvDownloadLink {
    pub accounting_csv_id: AccountingCsvId,
    pub link: AccountingCsvDownloadLink,
}
