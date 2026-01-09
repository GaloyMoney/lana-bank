use es_entity::*;
use sqlx::PgPool;
use std::collections::HashMap;

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

    pub async fn find_all_by_visibility<Out: From<DomainConfig>>(
        &self,
        ids: &[DomainConfigId],
        visibility: Visibility,
    ) -> Result<HashMap<DomainConfigId, Out>, DomainConfigError> {
        let (entities, _) = es_entity::es_query!(
            tbl_prefix = "core",
            "SELECT id FROM core_domain_configs WHERE id = ANY($1) AND visibility = $2",
            ids as &[DomainConfigId],
            visibility as Visibility,
        )
        .fetch_n(self.pool(), ids.len())
        .await?;

        Ok(entities
            .into_iter()
            .map(|entity| (entity.id, Out::from(entity)))
            .collect())
    }
}
