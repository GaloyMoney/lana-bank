use es_entity::*;
use sqlx::PgPool;

use crate::{
    entity::{DomainConfig, DomainConfigEvent},
    error::DomainConfigError,
    primitives::{DomainConfigId, DomainConfigKey},
};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DomainConfig",
    id = "DomainConfigId",
    err = "DomainConfigError",
    columns(key(ty = "DomainConfigKey", list_by)),
    tbl_prefix = "core"
)]
pub struct DomainConfigRepo {
    pool: PgPool,
}

impl DomainConfigRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
