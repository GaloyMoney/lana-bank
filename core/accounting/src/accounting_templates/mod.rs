use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::primitives::{AccountingTemplateId, CoreAccountingAction, CoreAccountingObject};

mod entity;
pub mod error;
mod repo;

pub use entity::*;
pub use error::AccountingTemplateError;
pub use repo::AccountingTemplateRepo;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountingTemplateEntry {
    pub account_id_or_code: String,
    pub direction: crate::primitives::DebitOrCredit,
    pub description_template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct AccountingTemplateValues {
    pub chart_ref: Option<String>,
    pub description_template: String,
    pub entries: Vec<AccountingTemplateEntry>,
}

#[derive(Clone)]
pub struct AccountingTemplates<Perms>
where
    Perms: PermissionCheck,
{
    authz: Arc<Perms>,
    repo: Arc<AccountingTemplateRepo>,
}

impl<Perms> AccountingTemplates<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreAccountingAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreAccountingObject>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        let repo = AccountingTemplateRepo::new(pool, clock);
        Self {
            authz,
            repo: Arc::new(repo),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.accounting_template.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<AccountingTemplateId> + std::fmt::Debug + Copy,
    ) -> Result<Option<AccountingTemplate>, AccountingTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::accounting_template(id.into()),
                CoreAccountingAction::ACCOUNTING_TEMPLATE_READ,
            )
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(template) => Ok(Some(template)),
            Err(AccountingTemplateError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[record_error_severity]
    #[instrument(name = "core_accounting.accounting_template.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Vec<AccountingTemplate>, AccountingTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                CoreAccountingObject::all_accounting_templates(),
                CoreAccountingAction::ACCOUNTING_TEMPLATE_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_by_name(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }
}
