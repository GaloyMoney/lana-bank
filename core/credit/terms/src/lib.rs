#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod balance_summary;
pub mod collateralization;
mod cvl;
mod effective_date;
mod error;
pub mod primitives;
pub mod terms_template;
mod value;

pub use cvl::CVLPct;
pub use effective_date::EffectiveDate;
pub use error::TermsError;
pub use primitives::{
    CoreTermsAction, CoreTermsObject, TermsPermissions, TermsTemplateAction,
    TermsTemplateAllOrOne, PERMISSION_SET_CREDIT_TERM_TEMPLATES, PERMISSION_SET_TERMS_VIEWER,
};
pub use terms_template::{
    NewTermsTemplate, NewTermsTemplateBuilder, TermsTemplate, TermsTemplateBuilder,
    TermsTemplateError, TermsTemplateEvent, TermsTemplateId, TermsTemplatePermissions,
    TermsTemplateRepo, TermsTemplates,
};
pub use value::{
    AnnualRatePct, DisbursalPolicy, FacilityDuration, FacilityDurationType, InterestInterval,
    InterestPeriod, ObligationDuration, OneTimeFeeRatePct, TermValues, TermValuesBuilder,
};
