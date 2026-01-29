pub mod entity;
pub mod error;
pub mod repo;
mod service;

es_entity::entity_id! { TermsTemplateId }

pub use entity::*;
pub use error::TermsTemplateError;
pub use repo::TermsTemplateRepo;
pub use service::{TermsTemplatePermissions, TermsTemplates};
