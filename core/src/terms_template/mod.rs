mod entity;
pub mod error;
mod repo;

use crate::{
    audit::Audit,
    authorization::{Authorization, Object, TermAction},
    data_export::Export,
    loan::TermValues,
    primitives::{LoanTermsId, Subject},
};

pub use entity::*;
use error::TermsTemplateError;
pub use repo::TermsTemplateRepo;

#[derive(Clone)]
pub struct TermsTemplates {
    pool: sqlx::PgPool,
    authz: Authorization,
    audit: Audit,
    repo: TermsTemplateRepo,
}

impl TermsTemplates {
    pub fn new(pool: &sqlx::PgPool, authz: &Authorization, audit: &Audit, export: &Export) -> Self {
        let repo = TermsTemplateRepo::new(pool, export);
        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            audit: audit.clone(),
            repo,
        }
    }

    pub fn repo(&self) -> &TermsTemplateRepo {
        &self.repo
    }

    pub async fn create_terms_template(
        &self,
        sub: &Subject,
        name: String,
        values: TermValues,
    ) -> Result<TermsTemplate, TermsTemplateError> {
        // TODO change this to Create terms template
        let audit_info = self.authz
            .check_permission(sub, Object::Term, TermAction::Update)
            .await?;
        let new_terms_template = NewTermsTemplate::builder()
            .id(LoanTermsId::new())
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
        id: LoanTermsId,
    ) -> Result<Option<TermsTemplate>, TermsTemplateError> {
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
        self.repo.list().await
    }
}
