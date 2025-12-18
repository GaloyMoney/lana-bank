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
        let config = self.repo.find_by_key(T::KEY).await?;

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

    #[instrument(name = "domain_config.create_simple", skip(self, value), err)]
    pub async fn create_simple<T: SimpleScalar>(
        &self,
        config: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        if self
            .repo
            .maybe_find_by_key(config.key().clone())
            .await?
            .is_some()
        {
            return Err(DomainConfigError::InvalidState(format!(
                "Domain config {} already exists",
                config.key()
            )));
        }

        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_simple_value(
                domain_config_id,
                config.key().clone(),
                T::SIMPLE_TYPE,
                value.to_json(),
            )?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.update_simple", skip(self, value), err)]
    pub async fn update_simple<T: SimpleScalar>(
        &self,
        config: SimpleConfig<T>,
        value: T,
    ) -> Result<(), DomainConfigError> {
        let mut config_entity = self.repo.find_by_key(config.key().clone()).await?;
        if config_entity.update_simple(value)?.did_execute() {
            self.repo.update(&mut config_entity).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.get_simple", skip(self), err)]
    pub async fn get_simple<T: SimpleScalar>(
        &self,
        config: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let config_entity = self.repo.find_by_key(config.key().clone()).await?;
        config_entity.current_simple_value::<T>()
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
            acc.push(config.to_simple_entry(simple_type)?);
        }
        next = ret.into_next_query();
    }

    Ok(())
}
