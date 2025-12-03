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

        Ok(config.current_value()?)
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
        let value_json = serde_json::to_value(value.clone())?;
        let new = NewDomainConfig::builder()
            .id(domain_config_id)
            .key(T::KEY)
            .value(value_json)
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create_in_op(op, new).await?;

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
        let value_json = serde_json::to_value(value.clone())?;

        let mut config = self.repo.find_by_key(T::KEY).await?;

        config.apply_update(value_json);
        self.repo.update_in_op(op, &mut config).await?;

        Ok(())
    }
}
