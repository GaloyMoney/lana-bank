pub mod error;
mod repo;
pub mod template;
mod value;

use crate::{
    authorization::{Authorization, Object, TermsTemplateAction},
    data_export::Export,
    primitives::{LoanTermsId, Subject},
};

use error::TermsTemplateError;
use repo::TermsTemplateRepo;
pub use template::*;
pub use value::*;

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
            .check_permission(sub, Object::TermsTemplate, TermsTemplateAction::Create)
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
        self.authz
            .check_permission(sub, Object::TermsTemplate, TermsTemplateAction::Read)
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
            .check_permission(sub, Object::TermsTemplate, TermsTemplateAction::List)
            .await?;
        self.repo.list().await
    }
}
