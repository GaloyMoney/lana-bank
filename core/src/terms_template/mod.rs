pub mod entity;
pub mod error;
mod repo;

use crate::{
    authorization::{Authorization, Object, TermsTemplateAction},
    data_export::Export,
    primitives::{Subject, TermsTemplateId},
    terms::TermValues,
};

pub use entity::*;
use error::TermsTemplateError;
use repo::TermsTemplateRepo;

#[derive(Clone)]
pub struct TermsTemplates {
    pool: sqlx::PgPool,
    authz: Authorization,
    repo: TermsTemplateRepo,
}

impl TermsTemplates {
    pub fn new(pool: &sqlx::PgPool, authz: &Authorization, export: &Export) -> Self {
        let repo = TermsTemplateRepo::new(pool, export);
        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            repo,
        }
    }

    pub async fn create_terms_template(
        &self,
        sub: &Subject,
        name: String,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::TermsTemplate, TermsTemplateAction::Create)
            .await?;
        let new_terms_template = NewTermsTemplate::builder()
            .id(TermsTemplateId::new())
            .name(name)
            .values(values)
            .audit_info(audit_info)
            .build()
            .expect("Could not build TermsTemplate");

        let mut db = self.pool.begin().await?;
        let terms_template = self.repo.create_in_tx(&mut db, new_terms_template).await?;
        db.commit().await?;
        Ok(terms_template)
    }

    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: TermsTemplateId,
    ) -> Result<Option<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(sub, Object::TermsTemplate, TermsTemplateAction::Read)
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(template) => Ok(Some(template)),
            Err(TermsTemplateError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_terms_templates(
        &self,
        sub: &Subject,
    ) -> Result<Vec<TermsTemplate>, TermsTemplateError> {
        self.authz
            .enforce_permission(sub, Object::TermsTemplate, TermsTemplateAction::List)
            .await?;
        self.repo.list().await
    }

    pub async fn update_term_values(
        &self,
        sub: &Subject,
        id: TermsTemplateId,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::TermsTemplate, TermsTemplateAction::Update)
            .await?;

        let mut terms_template = self.repo.find_by_id(id).await?;
        terms_template.update_values(values, audit_info);

        let mut db = self.pool.begin().await?;
        self.repo
            .persist_in_tx(&mut db, &mut terms_template)
            .await?;

        db.commit().await?;

        Ok(terms_template)
    }
}
