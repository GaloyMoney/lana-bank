#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;
mod simple;

use tracing::instrument;

pub use entity::{DomainConfig, DomainConfigEvent, NewDomainConfig};
pub use error::DomainConfigError;
pub use primitives::{DomainConfigId, DomainConfigKey, DomainConfigValue};
pub use simple::{SimpleConfig, SimpleEntry, SimpleScalar, SimpleType, SimpleValue};

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
        let key = T::KEY;
        let config = self.repo.find_by_key(key).await?;
        config.current_value()
    }

    #[instrument(name = "domain_config.get_or_default", skip(self), err)]
    pub async fn get_or_default<T>(&self) -> Result<T, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let maybe_config = self.repo.maybe_find_by_key(T::KEY).await?;
        let config_value = match maybe_config {
            Some(config) => config.current_value()?,
            None => T::default(),
        };

        Ok(config_value)
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

    #[instrument(name = "domain_config.update", skip(self, value), err)]
    pub async fn update<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let key = T::KEY;
        let mut config = self.repo.find_by_key(key.clone()).await?;

        if config.update(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.upsert", skip(self, value), err)]
    pub async fn upsert<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        let key = T::KEY;
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            self.update(value).await
        } else {
            self.create(value).await
        }
    }

    #[instrument(name = "domain_config.list_simple", skip(self), err)]
    pub async fn list_simple(&self) -> Result<Vec<SimpleEntry>, DomainConfigError> {
        let mut entries = Vec::new();
        for simple_type in [
            SimpleType::Bool,
            SimpleType::String,
            SimpleType::Int,
            SimpleType::Decimal,
        ] {
            let ret = self
                .repo
                .list_for_simple_type_by_created_at(
                    Some(simple_type),
                    es_entity::PaginatedQueryArgs {
                        first: usize::MAX,
                        after: None,
                    },
                    Default::default(),
                )
                .await?;
            for config in ret.entities {
                entries.push(config.into_simple_entry()?);
            }
        }

        Ok(entries)
    }

    #[instrument(name = "domain_config.get_simple", skip(self), err)]
    pub async fn get_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        let config = self.repo.find_by_key(key).await?;
        config.current_simple_value::<T>()
    }

    #[instrument(name = "domain_config.create_simple", skip(self, value), err)]
    pub async fn create_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_simple_value(domain_config_id, spec, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.update_simple", skip(self, value), err)]
    pub async fn update_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        let mut config = self.repo.find_by_key(key.clone()).await?;
        if config.update_simple(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.upsert_simple", skip(self, value), err)]
    pub async fn upsert_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key: DomainConfigKey = DomainConfigKey::from(spec.key);
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            self.update_simple(spec, value).await
        } else {
            self.create_simple(spec, value).await
        }
    }
}
