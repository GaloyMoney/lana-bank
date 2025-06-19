use serde::{Deserialize, Serialize};

es_entity::entity_id! {
    DocumentId,
    ReferenceId,
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DocumentType {
    CustomerDocument,
    LedgerAccountCsv,
}
