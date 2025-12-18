#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;
mod spec;
mod simple;

use tracing::instrument;

pub use entity::{DomainConfig, DomainConfigEvent, NewDomainConfig};
pub use error::DomainConfigError;
pub use primitives::{DomainConfigId, DomainConfigKey, DomainConfigValue};
pub use spec::{ConfigKind, ConfigSpec, TypedConfig};
pub use simple::{SimpleConfig, SimpleEntry, SimpleScalar, SimpleType, SimpleValue};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DomainConfigEvent;
}

use repo::DomainConfigRepo;
use spec::{ConfigSpec, ConfigSpecAdapter, TypedConfig};

#[derive(Clone)]
pub struct DomainConfigs {
    repo: DomainConfigRepo,
}

impl DomainConfigs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo }
    }

    #[instrument(name = "domain_config.create", skip(self, value), err)]
    pub async fn create<S>(&self, spec: S, value: S::Value) -> Result<(), DomainConfigError>
    where
        S: ConfigSpec + ConfigSpecAdapter + Copy,
    {
        match S::kind() {
            ConfigKind::Simple(simple_type) => {
                let handle = spec
                    .as_simple()
                    .expect("spec of kind Simple must be SimpleConfig");
                self.create_simple_value(handle, simple_type, value).await
            }
            ConfigKind::Complex => {
                let handle = spec
                    .as_complex()
                    .expect("spec of kind Complex must be TypedConfig");
                self.create_complex_value(handle, value).await
            }
        }
    }

    #[instrument(name = "domain_config.update", skip(self, value), err)]
    pub async fn update<S>(&self, spec: S, value: S::Value) -> Result<(), DomainConfigError>
    where
        S: ConfigSpec + ConfigSpecAdapter + Copy,
    {
        match S::kind() {
            ConfigKind::Simple(simple_type) => {
                let handle = spec
                    .as_simple()
                    .expect("spec of kind Simple must be SimpleConfig");
                self.update_simple_value(handle, simple_type, value).await
            }
            ConfigKind::Complex => {
                let handle = spec
                    .as_complex()
                    .expect("spec of kind Complex must be TypedConfig");
                self.update_complex_value(handle, value).await
            }
        }
    }

    #[instrument(name = "domain_config.get", skip(self), err)]
    pub async fn get<S>(&self, spec: S) -> Result<S::Value, DomainConfigError>
    where
        S: ConfigSpec + ConfigSpecAdapter + Copy,
    {
        match S::kind() {
            ConfigKind::Simple(_) => {
                let handle = spec
                    .as_simple()
                    .expect("spec of kind Simple must be SimpleConfig");
                self.get_simple_value(handle).await
            }
            ConfigKind::Complex => {
                let handle = spec
                    .as_complex()
                    .expect("spec of kind Complex must be TypedConfig");
                self.get_complex_value(handle).await
            }
        }
    }

    #[instrument(name = "domain_config.upsert", skip(self, value), err)]
    pub async fn upsert<S>(&self, spec: S, value: S::Value) -> Result<(), DomainConfigError>
    where
        S: ConfigSpec + ConfigSpecAdapter + Copy,
    {
        let key = spec.key();
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            self.update(spec, value).await
        } else {
            self.create(spec, value).await
        }
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

    #[instrument(name = "domain_config.list_simple", skip(self), err)]
    pub async fn list_simple(&self) -> Result<Vec<SimpleEntry>, DomainConfigError> {
        let mut entries = Vec::new();
        let mut next = Some(es_entity::PaginatedQueryArgs::default());

        while let Some(query) = next.take() {
            let mut ret = self
                .repo
                .list_for_simple_type_by_created_at(None, query, Default::default())
                .await?;
            for config in &ret.entities {
                entries.push(config.to_simple_entry()?);
            }
            next = ret.into_next_query();
        }

        Ok(entries)
    }

    async fn create_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        simple_type: SimpleType,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key = spec.key();
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            return Err(DomainConfigError::InvalidState(format!(
                "Domain config {} already exists",
                key
            )));
        }

        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_simple_value(domain_config_id, key, simple_type, value.to_json())?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    async fn update_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        expected: SimpleType,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key = spec.key();
        let mut config = self.repo.find_by_key(key.clone()).await?;
        config.ensure_simple_type(expected)?;
        if config.update_simple(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    async fn get_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let key = spec.key();
        let config = self.repo.find_by_key(key).await?;
        config.current_simple_value::<T>()
    }

    async fn create_complex_value<T: DomainConfigValue>(
        &self,
        _spec: TypedConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key = T::KEY;

        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            return Err(DomainConfigError::InvalidState(format!(
                "Domain config {} already exists",
                key
            )));
        }

        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    async fn update_complex_value<T: DomainConfigValue>(
        &self,
        _spec: TypedConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key = T::KEY;
        let mut config = self.repo.find_by_key(key.clone()).await?;
        config.ensure_complex()?;

        if config.update(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    async fn get_complex_value<T: DomainConfigValue>(
        &self,
        _spec: TypedConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let key = T::KEY;
        let config = self.repo.find_by_key(key).await?;
        config.ensure_complex()?;
        config.current_value()
    }
}
