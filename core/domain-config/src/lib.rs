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

    #[instrument(name = "domain_config.create", skip(self, value), err)]
    pub async fn create<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        self.create_complex_value(value).await
    }

    #[instrument(name = "domain_config.update", skip(self, value), err)]
    pub async fn update<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: DomainConfigValue,
    {
        self.update_complex_value(value).await
    }

    #[instrument(name = "domain_config.get", skip(self), err)]
    pub async fn get<T>(&self) -> Result<T, DomainConfigError>
    where
        T: DomainConfigValue,
    {
        self.get_complex_value::<T>().await
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
        for simple_type in [
            SimpleType::Bool,
            SimpleType::String,
            SimpleType::Int,
            SimpleType::Decimal,
        ] {
            collect_simple_of_type(&self.repo, simple_type, &mut entries).await?;
        }

        Ok(entries)
    }

    #[instrument(name = "domain_config.create_simple", skip(self, value), err)]
    pub async fn create_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        self.create_simple_value(spec, value).await
    }

    #[instrument(name = "domain_config.update_simple", skip(self, value), err)]
    pub async fn update_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        self.update_simple_value(spec, value).await
    }

    #[instrument(name = "domain_config.get_simple", skip(self), err)]
    pub async fn get_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        self.get_simple_value(spec).await
    }

    #[instrument(name = "domain_config.upsert_simple", skip(self, value), err)]
    pub async fn upsert_simple<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            self.update_simple(spec, value).await
        } else {
            self.create_simple(spec, value).await
        }
    }

    async fn create_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        if self.repo.maybe_find_by_key(key.clone()).await?.is_some() {
            return Err(DomainConfigError::InvalidState(format!(
                "Domain config {} already exists",
                key
            )));
        }

        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_simple_value(domain_config_id, key, T::SIMPLE_TYPE, value.to_json())?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    async fn update_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        let mut config = self.repo.find_by_key(key.clone()).await?;
        config.ensure_simple_type(T::SIMPLE_TYPE)?;
        if config.update_simple(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    async fn get_simple_value<T: SimpleScalar>(
        &self,
        spec: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let key: DomainConfigKey = spec.into();
        let config = self.repo.find_by_key(key).await?;
        config.current_simple_value::<T>()
    }

    async fn create_complex_value<T: DomainConfigValue>(
        &self,
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
    ) -> Result<T, DomainConfigError> {
        let key = T::KEY;
        let config = self.repo.find_by_key(key).await?;
        config.ensure_complex()?;
        config.current_value()
    }
}

async fn collect_simple_of_type(
    repo: &DomainConfigRepo,
    simple_type: SimpleType,
    acc: &mut Vec<SimpleEntry>,
) -> Result<(), DomainConfigError> {
    let mut next = Some(es_entity::PaginatedQueryArgs::default());

    while let Some(query) = next.take() {
        let mut ret = repo
            .list_for_simple_type_by_created_at(Some(simple_type), query, Default::default())
            .await?;
        for config in &ret.entities {
            acc.push(config.to_simple_entry()?);
        }
        next = ret.into_next_query();
    }

    Ok(())
}
