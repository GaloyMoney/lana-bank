use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "User",
    err = "UserError",
    columns(
        email(ty = "String", list_by),
        authentication_id(ty = "Option<AuthenticationId>", list_by, create(persist = false)),
    ),
    tbl_prefix = "core",
)]
pub(crate) struct UserRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
