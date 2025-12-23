#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use tracing::instrument;

pub use entity::{DomainConfig, DomainConfigEvent, NewDomainConfig};
pub use error::DomainConfigError;
pub use primitives::{DomainConfigId, DomainConfigKey, DomainConfigValue};

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
    pub async fn get<T>(&self) -> Result<T, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let config = match self.repo.find_by_key(T::KEY).await {
            Err(e) if e.was_not_found() => Err(DomainConfigError::NotConfigured),
            Err(e) => Err(e),
            Ok(config) => Ok(config),
        }?;

        config.current_value()
    }

    #[instrument(name = "domain_config.get_or_default", skip(self), err)]
    pub async fn get_or_default<T>(&self) -> Result<T, DomainConfigError>
    where
        T: DomainConfigValue + Default,
    {
        let maybe_config = self.repo.maybe_find_by_key(T::KEY).await?;
        let config_value = match maybe_config {
            Some(config) => config.current_value()?,
            None => T::default(),
        };

        Ok(config_value)
    }

    #[instrument(name = "domain_config.create_in_op", skip(self, op, value), err)]
    pub async fn create_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create_in_op(op, new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.create", skip(self, value), err)]
    pub async fn create<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.update_in_op", skip(self, op, value), err)]
    pub async fn update_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let mut config = self.repo.find_by_key_in_op(&mut *op, T::KEY).await?;

        if config.update(value)?.did_execute() {
            self.repo.update_in_op(op, &mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.update", skip(self, value), err)]
    pub async fn update<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let mut config = self.repo.find_by_key(T::KEY).await?;
        if config.update(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.upsert_in_op", skip(self, op, value), err)]
    pub async fn upsert_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        match self.update_in_op(op, value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create_in_op(op, value).await
            }
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "domain_config.upsert", skip(self, value), err)]
    pub async fn upsert<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        match self.update(value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create(value).await
            }
            Err(e) => Err(e),
        }
    }
}
