use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "TermsTemplate",
    err = "TermsTemplateError",
    columns(name(ty = "String", list_by))
)]
pub struct TermsTemplateRepo {
    pool: PgPool,
}

impl TermsTemplateRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
