pub mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;

use crate::{CoreCreditAction, CoreCreditObject, TermValues, primitives::TermsTemplateId};

pub use entity::*;

#[cfg(feature = "json-schema")]
pub use entity::TermsTemplateEvent;
use error::TermsTemplateError;
use repo::TermsTemplateRepo;

#[derive(Clone)]
pub struct TermsTemplates<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    repo: TermsTemplateRepo,
}

impl<Perms> TermsTemplates<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let repo = TermsTemplateRepo::new(pool);
        Self {
            authz: authz.clone(),
            repo,
        }
    }

    pub async fn subject_can_create_terms_template(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<audit::AuditInfo>, TermsTemplateError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_terms_templates(),
                CoreCreditAction::TERMS_TEMPLATE_CREATE,
                enforce,
            )
            .await?)
    }

    pub async fn create_terms_template(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        self.subject_can_create_terms_template(sub, true)
            .await?
            .expect("audit info missing");
        let new_terms_template = NewTermsTemplate::builder()
            .id(TermsTemplateId::new())
            .name(name)
            .values(values)
            .build()
            .expect("Could not build TermsTemplate");

        let terms_template = self.repo.create(new_terms_template).await?;
        Ok(terms_template)
    }

    pub async fn subject_can_update_terms_template(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<audit::AuditInfo>, TermsTemplateError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_terms_templates(),
                CoreCreditAction::TERMS_TEMPLATE_UPDATE,
                enforce,
            )
            .await?)
    }

    pub async fn update_term_values(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: TermsTemplateId,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        self.subject_can_update_terms_template(sub, true)
            .await?
            .expect("audit info missing");

        let mut terms_template = self.repo.find_by_id(id).await?;
        terms_template.update_values(values);

        self.repo.update(&mut terms_template).await?;

        Ok(terms_template)
    }

    #[instrument(name = "core_credit.terms_template.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<TermsTemplateId> + std::fmt::Debug + Copy,
    ) -> Result<Option<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::terms_template(id.into()),
                CoreCreditAction::TERMS_TEMPLATE_READ,
            )
            .await?;
        match self.repo.find_by_id(id.into()).await {
            Ok(template) => Ok(Some(template)),
            Err(TermsTemplateError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Vec<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_terms_templates(),
                CoreCreditAction::TERMS_TEMPLATE_LIST,
            )
            .await?;
        Ok(self
            .repo
            .list_by_name(Default::default(), es_entity::ListDirection::Ascending)
            .await?
            .entities)
    }

    pub async fn find_all<T: From<TermsTemplate>>(
        &self,
        ids: &[TermsTemplateId],
    ) -> Result<HashMap<TermsTemplateId, T>, TermsTemplateError> {
        self.repo.find_all(ids).await
    }
}
