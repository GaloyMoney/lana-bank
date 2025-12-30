use es_entity::*;
use sqlx::PgPool;

use crate::{
    entity::{DomainConfig, DomainConfigEvent},
    error::DomainConfigError,
    primitives::{DomainConfigId, DomainConfigKey, Visibility},
};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DomainConfig",
    id = "DomainConfigId",
    err = "DomainConfigError",
    columns(
        key(ty = "DomainConfigKey", list_by),
        visibility(
            ty = "Visibility",
            list_for,
            create(accessor = "visibility"),
            update(persist = false)
        )
    ),
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
