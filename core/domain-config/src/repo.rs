use es_entity::*;
use sqlx::PgPool;
use tracing::instrument;

use crate::{
    SimpleType,
    entity::{DomainConfig, DomainConfigEvent},
    error::DomainConfigError,
    primitives::{DomainConfigId, DomainConfigKey},
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
            list_for,
            create(accessor = "simple_type"),
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

    #[instrument(name = "domain_config_repo.list_simple_in_op", skip(self, op), err)]
    pub async fn list_simple_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        cursor: es_entity::PaginatedQueryArgs<domain_config_cursor::DomainConfigsByKeyCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DomainConfig, domain_config_cursor::DomainConfigsByKeyCursor>,
        DomainConfigError,
    > {
        let es_entity::PaginatedQueryArgs { first, after } = cursor;
        let key = after.map(|after| after.key);
        let limit = (first + 1) as i64;

        let (entities, has_next_page) = es_entity::es_query!(
            tbl_prefix = "core",
            r#"
                SELECT id, key, simple_type
                FROM core_domain_configs
                WHERE simple_type IS NOT NULL
                  AND ($2::text IS NULL OR key > $2::text)
                ORDER BY key ASC
                LIMIT $1
            "#,
            limit,
            key as Option<DomainConfigKey>,
        )
        .fetch_n(op, first)
        .await?;

        let end_cursor = entities
            .last()
            .map(domain_config_cursor::DomainConfigsByKeyCursor::from);

        Ok(es_entity::PaginatedQueryRet {
            entities,
            has_next_page,
            end_cursor,
        })
    }
}
