use es_entity::*;
use sqlx::PgPool;
use std::collections::HashMap;

use encryption::EncryptionConfig;

use crate::{
    entity::{DomainConfig, DomainConfigEvent},
    error::{DomainConfigError, DomainConfigHydrateError},
    primitives::{DomainConfigId, DomainConfigKey, Visibility},
};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DomainConfig",
    id = "DomainConfigId",
    columns(
        key(ty = "DomainConfigKey", list_by),
        visibility(ty = "Visibility", list_for(by(key)), update(persist = false)),
        encrypted(ty = "bool", list_for(by(id)), update(persist = false))
    ),
    tbl_prefix = "core",
    post_hydrate_hook(method = "validate_decryptable", error = "DomainConfigHydrateError")
)]
pub struct DomainConfigRepo {
    pool: PgPool,
    encryption_config: EncryptionConfig,
}

impl DomainConfigRepo {
    pub fn new(pool: &PgPool, encryption_config: &EncryptionConfig) -> Self {
        Self {
            pool: pool.clone(),
            encryption_config: encryption_config.clone(),
        }
    }

    fn validate_decryptable(&self, entity: &DomainConfig) -> Result<(), DomainConfigHydrateError> {
        entity.assert_decryptable(
            &self.encryption_config.encryption_key,
            self.encryption_config.deprecated_encryption_key.as_ref(),
        )
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
}
