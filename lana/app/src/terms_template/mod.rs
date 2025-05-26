pub mod entity;
pub mod error;
mod repo;

use std::collections::HashMap;

use authz::PermissionCheck;
use tracing::instrument;

use crate::{
    audit::AuditInfo,
    authorization::{Authorization, Object, TermsTemplateAction},
    primitives::{Subject, TermsTemplateId},
    terms::TermValues,
};

pub use entity::*;
use error::TermsTemplateError;
use repo::TermsTemplateRepo;

#[derive(Clone)]
pub struct TermsTemplates {
    authz: Authorization,
    repo: TermsTemplateRepo,
}

impl TermsTemplates {
    pub fn new(pool: &sqlx::PgPool, authz: &Authorization) -> Self {
        let repo = TermsTemplateRepo::new(pool);
        Self {
            authz: authz.clone(),
            repo,
        }
    }

    pub async fn subject_can_create_terms_template(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, TermsTemplateError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::all_terms_templates(),
                TermsTemplateAction::Create,
                enforce,
            )
            .await?)
    }

    pub async fn create_terms_template(
        &self,
        sub: &Subject,
        name: String,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        let audit_info = self
            .subject_can_create_terms_template(sub, true)
            .await?
            .expect("audit info missing");
        let new_terms_template = NewTermsTemplate::builder()
            .id(TermsTemplateId::new())
            .name(name)
            .values(values)
            .audit_info(audit_info)
            .build()
            .expect("Could not build TermsTemplate");

        let terms_template = self.repo.create(new_terms_template).await?;
        Ok(terms_template)
    }

    pub async fn subject_can_update_terms_template(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, TermsTemplateError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::all_terms_templates(),
                TermsTemplateAction::Update,
                enforce,
            )
            .await?)
    }

    pub async fn update_term_values(
        &self,
        sub: &Subject,
        id: TermsTemplateId,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        let audit_info = self
            .subject_can_update_terms_template(sub, true)
            .await?
            .expect("audit info missing");

        let mut terms_template = self.repo.find_by_id(id).await?;
        terms_template.update_values(values, audit_info);

        self.repo.update(&mut terms_template).await?;

        Ok(terms_template)
    }

    #[instrument(name = "terms_template::find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<TermsTemplateId> + std::fmt::Debug + Copy,
    ) -> Result<Option<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                Object::terms_template(id.into()),
                TermsTemplateAction::Read,
            )
            .await?;
        match self.repo.find_by_id(id.into()).await {
            Ok(template) => Ok(Some(template)),
            Err(TermsTemplateError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(&self, sub: &Subject) -> Result<Vec<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                Object::all_terms_templates(),
                TermsTemplateAction::List,
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
