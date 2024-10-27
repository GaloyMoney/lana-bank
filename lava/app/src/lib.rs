#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod applicant;
pub mod authorization;
pub mod cli;
pub mod constants;
pub mod credit_facility;
pub mod customer;
pub mod data_export;
pub mod deposit;
pub mod document;
pub mod entity;
pub mod ledger;
pub mod loan;
pub mod price;
pub mod primitives;
pub mod report;
pub mod server;
pub mod service_account;
pub mod storage;
pub mod terms;
pub mod terms_template;
pub mod user;
pub mod withdraw;

pub mod job {
    pub use job::*;
}

pub mod audit {
    use crate::{
        authorization::{Action, Object},
        primitives::Subject,
    };

    pub use audit::{error, AuditCursor, AuditEntryId, AuditInfo, AuditSvc};
    pub type Audit = audit::Audit<Subject, Object, Action>;
    pub type AuditEntry = audit::AuditEntry<Subject, Object, Action>;
}
