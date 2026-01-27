pub mod error;

// Re-export value types from the extracted terms crate
// Note: CVLPct, EffectiveDate, CollateralizationRatio, CollateralizationState, and balance summary
// types are re-exported from primitives
pub use core_credit_terms::{
    AnnualRatePct, DisbursalPolicy, FacilityDuration, FacilityDurationType, InterestInterval,
    InterestPeriod, ObligationDuration, OneTimeFeeRatePct, TermValues, TermValuesBuilder,
};

use crate::primitives::DisbursedReceivableAccountCategory;

impl From<FacilityDurationType> for DisbursedReceivableAccountCategory {
    fn from(duration_type: FacilityDurationType) -> Self {
        match duration_type {
            FacilityDurationType::LongTerm => DisbursedReceivableAccountCategory::LongTerm,
            FacilityDurationType::ShortTerm => DisbursedReceivableAccountCategory::ShortTerm,
        }
    }
}
