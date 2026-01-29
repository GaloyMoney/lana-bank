use std::{collections::HashMap, sync::Arc};

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use super::{
    NewTermsTemplate, TermsTemplate, TermsTemplateError, TermsTemplateId, TermsTemplateRepo,
};
use crate::TermValues;

/// Trait for providing TermsTemplate-related authorization objects and actions.
/// Implement this trait in the consuming crate to provide the concrete types.
pub trait TermsTemplatePermissions {
    type Action: Clone + Send + Sync + std::fmt::Debug;
    type Object: Clone + Send + Sync + std::fmt::Debug;

    fn terms_template_create_action() -> Self::Action;
    fn terms_template_read_action() -> Self::Action;
    fn terms_template_update_action() -> Self::Action;
    fn terms_template_list_action() -> Self::Action;

    fn all_terms_templates_object() -> Self::Object;
    fn terms_template_object(id: TermsTemplateId) -> Self::Object;
}

#[derive(Clone)]
pub struct TermsTemplates<Perms, P>
where
    Perms: PermissionCheck,
    P: TermsTemplatePermissions,
{
    authz: Arc<Perms>,
    repo: Arc<TermsTemplateRepo>,
    _phantom: std::marker::PhantomData<P>,
}

impl<Perms, P> TermsTemplates<Perms, P>
where
    Perms: PermissionCheck,
    P: TermsTemplatePermissions,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<P::Action>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<P::Object>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        let repo = TermsTemplateRepo::new(pool, clock);
        Self {
            authz,
            repo: Arc::new(repo),
            _phantom: std::marker::PhantomData,
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
                P::all_terms_templates_object(),
                P::terms_template_create_action(),
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
                P::all_terms_templates_object(),
                P::terms_template_update_action(),
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

    #[record_error_severity]
    #[instrument(name = "core_credit_terms.terms_template.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<TermsTemplateId> + std::fmt::Debug + Copy,
    ) -> Result<Option<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(
                sub,
                P::terms_template_object(id.into()),
                P::terms_template_read_action(),
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
                P::all_terms_templates_object(),
                P::terms_template_list_action(),
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
