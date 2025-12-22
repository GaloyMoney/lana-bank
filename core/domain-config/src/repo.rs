use es_entity::*;
use sqlx::PgPool;

use crate::{
    entity::{DomainConfig, DomainConfigEvent},
    error::DomainConfigError,
    primitives::{ConfigType, DomainConfigId, DomainConfigKey},
};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DomainConfig",
    id = "DomainConfigId",
    err = "DomainConfigError",
    columns(
        key(ty = "DomainConfigKey", list_by),
        config_type(
            ty = "ConfigType",
            list_for,
            create(accessor = "config_type"),
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
