#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;
mod spec;

use tracing::instrument;

pub use entity::{DomainConfig, DomainConfigEvent, NewDomainConfig};
pub use error::DomainConfigError;
pub use primitives::{ConfigType, DomainConfigId, DomainConfigKey, Visibility};
pub use repo::domain_config_cursor::DomainConfigsByKeyCursor;
pub use spec::{Complex, ConfigSpec, Simple, ValueKind};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DomainConfigEvent;
}

use repo::DomainConfigRepo;

#[derive(Clone)]
pub struct DomainConfigs {
    repo: DomainConfigRepo,
}

impl DomainConfigs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo }
    }

    #[instrument(name = "domain_config.get", skip(self), err)]
    pub async fn get<C>(&self) -> Result<<C::Kind as ValueKind>::Value, DomainConfigError>
    where
        C: ConfigSpec,
    {
        let config = match self.repo.find_by_key(C::KEY).await {
            Err(e) if e.was_not_found() => Err(DomainConfigError::NotConfigured),
            Err(e) => Err(e),
            Ok(config) => Ok(config),
        }?;

        config.current_value::<C>()
    }

    #[instrument(name = "domain_config.get_or_default", skip(self), err)]
    pub async fn get_or_default<C>(
        &self,
    ) -> Result<<C::Kind as ValueKind>::Value, DomainConfigError>
    where
        C: ConfigSpec,
    {
        let maybe_config = self.repo.maybe_find_by_key(C::KEY).await?;
        match maybe_config {
            Some(config) => config.current_value::<C>(),
            None => {
                C::default_value().ok_or_else(|| DomainConfigError::NoDefault(C::KEY.to_string()))
            }
        }
    }

    #[instrument(name = "domain_config.create", skip(self, value), err)]
    pub async fn create<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value::<C>(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.update", skip(self, value), err)]
    pub async fn update<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ConfigSpec,
    {
        let mut config = self.repo.find_by_key(C::KEY).await?;
        if config.update_value::<C>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.upsert", skip(self, value), err)]
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
}
