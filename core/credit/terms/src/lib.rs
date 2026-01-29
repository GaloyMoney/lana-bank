#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod balance_summary;
pub mod collateralization;
mod cvl;
mod effective_date;
mod error;
pub mod terms_template;
mod value;

pub use cvl::CVLPct;
pub use effective_date::EffectiveDate;
pub use error::TermsError;
pub use terms_template::{
    NewTermsTemplate, NewTermsTemplateBuilder, TermsTemplate, TermsTemplateBuilder,
    TermsTemplateError, TermsTemplateEvent, TermsTemplateId, TermsTemplatePermissions,
    TermsTemplateRepo, TermsTemplates,
};
pub use value::{
    AnnualRatePct, DisbursalPolicy, FacilityDuration, FacilityDurationType, InterestInterval,
    InterestPeriod, ObligationDuration, OneTimeFeeRatePct, TermValues, TermValuesBuilder,
};
