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

use repo::{DomainConfigRepo, SimpleConfigCurrentRow};

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
            .simple_metadata_by_key(config.key())
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
        let Some(meta) = self.repo.simple_metadata_by_key(config.key()).await? else {
            return Err(DomainConfigError::EsEntityError(
                es_entity::EsEntityError::NotFound,
            ));
        };

        match meta.simple_type {
            Some(found) if found == T::SIMPLE_TYPE => {}
            Some(found) => {
                return Err(DomainConfigError::InvalidSimpleType {
                    key: config.key().clone(),
                    expected: T::SIMPLE_TYPE,
                    found: Some(found),
                });
            }
            None => {
                return Err(DomainConfigError::InvalidSimpleType {
                    key: config.key().clone(),
                    expected: T::SIMPLE_TYPE,
                    found: None,
                });
            }
        }

        let mut config_entity = self.repo.find_by_id(meta.id).await?;
        if config_entity
            .update_simple_value(value.to_json())
            .did_execute()
        {
            self.repo.update(&mut config_entity).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.get_simple", skip(self), err)]
    pub async fn get_simple<T: SimpleScalar>(
        &self,
        config: SimpleConfig<T>,
    ) -> Result<T, DomainConfigError> {
        let Some(meta) = self.repo.simple_metadata_by_key(config.key()).await? else {
            return Err(DomainConfigError::EsEntityError(
                es_entity::EsEntityError::NotFound,
            ));
        };

        match meta.simple_type {
            Some(found) if found == T::SIMPLE_TYPE => {}
            Some(found) => {
                return Err(DomainConfigError::InvalidSimpleType {
                    key: config.key().clone(),
                    expected: T::SIMPLE_TYPE,
                    found: Some(found),
                });
            }
            None => {
                return Err(DomainConfigError::InvalidSimpleType {
                    key: config.key().clone(),
                    expected: T::SIMPLE_TYPE,
                    found: None,
                });
            }
        }

        let Some(row) = self.repo.simple_with_value_by_key(config.key()).await? else {
            return Err(DomainConfigError::MissingSimpleValue(config.key().clone()));
        };

        T::from_json(row.value)
    }

    #[instrument(name = "domain_config.list_simple", skip(self), err)]
    pub async fn list_simple(&self) -> Result<Vec<SimpleEntry>, DomainConfigError> {
        let rows = self.repo.list_simple_with_values().await?;

        rows.into_iter().map(simple_entry_from_row).collect()
    }
}

fn simple_entry_from_row(row: SimpleConfigCurrentRow) -> Result<SimpleEntry, DomainConfigError> {
    let value = row.simple_type.parse_json(row.value)?;
    Ok(SimpleEntry {
        key: row.key.to_string(),
        simple_type: row.simple_type,
        value,
    })
}
