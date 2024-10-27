use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{CustomerId, DocumentId};

use super::{entity::*, error::DocumentError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Document",
    err = "DocumentError",
    columns(customer_id(ty = "CustomerId", list_for, update(persist = false))),
    delete = "soft"
)]
pub struct DocumentsRepo {
    pool: PgPool,
}

impl DocumentsRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
