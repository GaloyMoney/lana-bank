#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod macros;
mod primitives;
pub mod registry;
mod repo;
mod spec;
mod typed_domain_config;

use std::collections::{HashMap, HashSet};
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entity::DomainConfig;
pub use entity::DomainConfigEvent;
pub use error::DomainConfigError;
#[doc(hidden)]
pub use inventory;
pub use primitives::{ConfigType, DomainConfigId, DomainConfigKey, Visibility};
pub use repo::domain_config_cursor::DomainConfigsByKeyCursor;
pub use spec::{Complex, ConfigSpec, Simple, ValueKind};
pub use typed_domain_config::TypedDomainConfig;

use entity::NewDomainConfig;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DomainConfigEvent;
}

use repo::DomainConfigRepo;

#[derive(Clone)]
pub struct InternalDomainConfigs {
    repo: DomainConfigRepo,
}

#[derive(Clone)]
pub struct ExposedDomainConfigs {
    repo: DomainConfigRepo,
}

fn ensure_visibility<C: ConfigSpec>(expected: Visibility) -> Result<(), DomainConfigError> {
    if C::VISIBILITY != expected {
        return Err(DomainConfigError::InvalidState(format!(
            "Config {key} is {found}, expected {expected}",
            key = C::KEY,
            found = C::VISIBILITY,
            expected = expected,
        )));
    }
    Ok(())
}

fn ensure_internal<C: ConfigSpec>() -> Result<(), DomainConfigError> {
    ensure_visibility::<C>(Visibility::Internal)
}

fn ensure_exposed<C: ConfigSpec>() -> Result<(), DomainConfigError> {
    ensure_visibility::<C>(Visibility::Exposed)
}

impl InternalDomainConfigs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self))]
    pub async fn get<C>(&self) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_internal::<C>()?;
        let config = self.repo.find_by_key(C::KEY).await?;
        TypedDomainConfig::new(config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.create", skip(self, value))]
    pub async fn create<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_internal::<C>()?;
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value::<C>(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update", skip(self, value))]
    pub async fn update<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_internal::<C>()?;
        let mut config = self.repo.find_by_key(C::KEY).await?;
        if config.update_value::<C>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.upsert", skip(self, value))]
    pub async fn upsert<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
        <C::Kind as ValueKind>::Value: Clone,
    {
        match self.update::<C>(value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create::<C>(value).await
            }
            Err(e) => Err(e),
        }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.seed_registered", skip(self))]
    pub async fn seed_registered(&self) -> Result<(), DomainConfigError> {
        seed_registered_for_visibility(&self.repo, Visibility::Internal).await
    }
}

impl ExposedDomainConfigs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self))]
    pub async fn get<C>(&self) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_exposed::<C>()?;
        let config = self.repo.find_by_key(C::KEY).await?;
        TypedDomainConfig::new(config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.find_all_exposed", skip(self))]
    pub async fn find_all_exposed<T: From<DomainConfig>>(
        &self,
        ids: &[DomainConfigId],
    ) -> Result<HashMap<DomainConfigId, T>, DomainConfigError> {
        self.repo.find_all_exposed(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.create", skip(self, value))]
    pub async fn create<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_exposed::<C>()?;
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value::<C>(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update", skip(self, value))]
    pub async fn update<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        ensure_exposed::<C>()?;
        let mut config = self.repo.find_by_key(C::KEY).await?;
        if config.update_value::<C>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update_exposed_from_json", skip(self, value))]
    pub async fn update_exposed_from_json(
        &self,
        id: impl Into<DomainConfigId> + std::fmt::Debug,
        value: serde_json::Value,
    ) -> Result<DomainConfig, DomainConfigError> {
        let id = id.into();
        let mut config = self.repo.find_by_id(id).await?;
        let entry = registry::maybe_find_by_key(config.key.as_str()).ok_or_else(|| {
            DomainConfigError::InvalidKey(format!(
                "Registry entry missing for config key: {}",
                config.key
            ))
        })?;

        if config
            .apply_exposed_update_from_json(entry, value)?
            .did_execute()
        {
            self.repo.update(&mut config).await?;
        }

        Ok(config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.upsert", skip(self, value))]
    pub async fn upsert<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
        <C::Kind as ValueKind>::Value: Clone,
    {
        match self.update::<C>(value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create::<C>(value).await
            }
            Err(e) => Err(e),
        }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.list_exposed_configs", skip(self))]
    pub async fn list_exposed_configs(
        &self,
        query: es_entity::PaginatedQueryArgs<DomainConfigsByKeyCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DomainConfig, DomainConfigsByKeyCursor>,
        DomainConfigError,
    > {
        self.repo
            .list_for_visibility_by_key(
                Visibility::Exposed,
                query,
                es_entity::ListDirection::Ascending,
            )
            .await
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.seed_registered", skip(self))]
    pub async fn seed_registered(&self) -> Result<(), DomainConfigError> {
        seed_registered_for_visibility(&self.repo, Visibility::Exposed).await
    }
}

async fn seed_registered_for_visibility(
    repo: &DomainConfigRepo,
    visibility: Visibility,
) -> Result<(), DomainConfigError> {
    let mut seen = HashSet::new();
    for spec in registry::all_specs() {
        if !seen.insert(spec.key) {
            return Err(DomainConfigError::InvalidKey(format!(
                "Duplicate domain config key: {}",
                spec.key
            )));
        }

        if spec.visibility != visibility {
            continue;
        }

        let key = DomainConfigKey::new(spec.key);
        let config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .seed(config_id, key, spec.config_type, spec.visibility)
            .build()?;
        match repo.create(new).await {
            Ok(_) => {}
            Err(DomainConfigError::DuplicateKey) => continue,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
