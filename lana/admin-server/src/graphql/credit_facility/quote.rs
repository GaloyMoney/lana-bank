use async_graphql::*;

use crate::primitives::*;

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum CreditFacilityQuoteType {
    Disbursal,
    Interest,
}

#[derive(SimpleObject)]
pub struct CreditFacilityQuoteEntry {
    pub entry_type: CreditFacilityQuoteType,
    pub outstanding: UsdCents,
    pub due_at: Timestamp,
}

impl From<lana_app::credit::CreditFacilityQuoteEntry> for CreditFacilityQuoteEntry {
    fn from(entry: lana_app::credit::CreditFacilityQuoteEntry) -> Self {
        match entry {
            lana_app::credit::CreditFacilityQuoteEntry::Disbursal(entry) => Self {
                entry_type: CreditFacilityQuoteType::Disbursal,
                outstanding: entry.outstanding,
                due_at: entry.due_at.into(),
            },
            lana_app::credit::CreditFacilityQuoteEntry::Interest(entry) => Self {
                entry_type: CreditFacilityQuoteType::Interest,
                outstanding: entry.outstanding,
                due_at: entry.due_at.into(),
            },
        }
    }
}
