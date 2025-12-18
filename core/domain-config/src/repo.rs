use es_entity::*;
use sqlx::PgPool;

use crate::{
    entity::{DomainConfig, DomainConfigEvent},
    error::DomainConfigError,
    primitives::{DomainConfigId, DomainConfigKey},
    simple::SimpleType,
};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DomainConfig",
    id = "DomainConfigId",
    err = "DomainConfigError",
    columns(
        key(ty = "DomainConfigKey", list_by),
        simple_type(
            ty = "Option<SimpleType>",
            create(accessor = "simple_type()"),
            update(persist = false)
        )
    ),
    tbl_prefix = "core"
)]
pub struct DomainConfigRepo {
    pool: PgPool,
}

#[derive(Debug)]
pub(crate) struct SimpleConfigMeta {
    pub id: DomainConfigId,
    pub simple_type: Option<SimpleType>,
}

#[derive(Debug)]
pub(crate) struct SimpleConfigCurrentRow {
    pub key: DomainConfigKey,
    pub simple_type: SimpleType,
    pub value: serde_json::Value,
}

impl DomainConfigRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn simple_metadata_by_key(
        &self,
        key: &DomainConfigKey,
    ) -> Result<Option<SimpleConfigMeta>, DomainConfigError> {
        let meta = sqlx::query_as!(
            SimpleConfigMeta,
            r#"
            SELECT id, simple_type as "simple_type: Option<SimpleType>"
            FROM core_domain_configs
            WHERE key = $1
            "#,
            key as DomainConfigKey,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(meta)
    }

    pub async fn simple_with_value_by_key(
        &self,
        key: &DomainConfigKey,
    ) -> Result<Option<SimpleConfigCurrentRow>, DomainConfigError> {
        let row = sqlx::query_as!(
            SimpleConfigCurrentRow,
            r#"
            WITH latest AS (
              SELECT DISTINCT ON (id) id, value
              FROM core_domain_config_events_rollup
              ORDER BY id, version DESC
            )
            SELECT c.key as "key: DomainConfigKey", c.simple_type as "simple_type: SimpleType", l.value
            FROM core_domain_configs c
            JOIN latest l USING (id)
            WHERE c.simple_type IS NOT NULL
              AND c.key = $1
            "#,
            key as DomainConfigKey,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_simple_with_values(
        &self,
    ) -> Result<Vec<SimpleConfigCurrentRow>, DomainConfigError> {
        let rows = sqlx::query_as!(
            SimpleConfigCurrentRow,
            r#"
            WITH latest AS (
              SELECT DISTINCT ON (id) id, value
              FROM core_domain_config_events_rollup
              ORDER BY id, version DESC
            )
            SELECT c.key as "key: DomainConfigKey", c.simple_type as "simple_type: SimpleType", l.value
            FROM core_domain_configs c
            JOIN latest l USING (id)
            WHERE c.simple_type IS NOT NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
