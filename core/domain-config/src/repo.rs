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
            list_for(by(key)),
            update(persist = false)
        ),
        encrypted(ty = "bool", list_for(by(id)), update(persist = false))
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

    pub async fn find_all_exposed<Out: From<DomainConfig>>(
        &self,
        ids: &[DomainConfigId],
    ) -> Result<HashMap<DomainConfigId, Out>, DomainConfigError> {
        let (entities, _) = es_entity::es_query!(
            tbl_prefix = "core",
            "SELECT id FROM core_domain_configs WHERE id = ANY($1) AND visibility = 'exposed'",
            ids as &[DomainConfigId],
        )
        .fetch_n(self.pool(), ids.len())
        .await?;

        Ok(entities
            .into_iter()
            .map(|entity| (entity.id, Out::from(entity)))
            .collect())
    }

    pub async fn list_all_encrypted_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
    ) -> Result<Vec<DomainConfig>, DomainConfigError> {
        let mut next = Some(PaginatedQueryArgs::default());
        let mut domain_configs = Vec::new();
        while let Some(query) = next {
            let mut ret = self
                .list_for_encrypted_by_id_in_op(&mut *op, true, query, Default::default())
                .await?;
            domain_configs.append(&mut ret.entities);
            next = ret.into_next_query();
        }
        Ok(domain_configs)
    }

    #[tracing::instrument(name = "domain_config.update_all_encrypted_in_op", skip_all)]
    pub async fn update_all_encrypted_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        entities: &mut [DomainConfig],
    ) -> Result<(), DomainConfigError> {
        let ids: Vec<DomainConfigId> = entities.iter().map(|e| e.id).collect();
        sqlx::query!(
            r#"
            UPDATE core_domain_config_events
            SET event = jsonb_set(event, '{value}', 'null'::jsonb, false)
            WHERE id = ANY($1)
                AND event_type = 'updated'
                AND event->'value'->>'type' = 'encrypted'
            "#,
            &ids as &[DomainConfigId],
        )
        .execute(op.as_executor())
        .await?;
        self.update_all_in_op(op, entities).await?;

        Ok(())
    }
}
