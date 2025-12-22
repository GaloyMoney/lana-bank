#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod complex;
mod entity;
pub mod error;
mod primitives;
mod repo;
mod simple;

use tracing::instrument;

pub use complex::ComplexConfig;
pub use entity::{DomainConfig, DomainConfigEvent, NewDomainConfig};
pub use error::DomainConfigError;
pub use primitives::{ConfigType, DomainConfigId, DomainConfigKey};
pub use repo::domain_config_cursor::DomainConfigsByKeyCursor;
pub use simple::{SimpleConfig, SimpleEntry, SimpleScalar};

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

    #[instrument(name = "domain_config.get_complex", skip(self), err)]
    pub async fn get_complex<T>(&self) -> Result<T, DomainConfigError>
    where
        T: ComplexConfig,
    {
        let config = self.repo.find_by_key(T::KEY).await?;

        config.current_complex_value()
    }

    #[instrument(name = "domain_config.get_complex_or_default", skip(self), err)]
    pub async fn get_complex_or_default<T>(&self) -> Result<T, DomainConfigError>
    where
        T: ComplexConfig,
    {
        let maybe_config = self.repo.maybe_find_by_key(T::KEY).await?;
        let config_value = match maybe_config {
            Some(config) => config.current_complex_value()?,
            None => T::default(),
        };

        Ok(config_value)
    }

    #[instrument(
        name = "domain_config.create_complex_in_op",
        skip(self, op, value),
        err
    )]
    pub async fn create_complex_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_complex_value(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create_in_op(op, new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.create_complex", skip(self, value), err)]
    pub async fn create_complex<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_complex_value(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(
        name = "domain_config.update_complex_in_op",
        skip(self, op, value),
        err
    )]
    pub async fn update_complex_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        let mut config = self.repo.find_by_key_in_op(&mut *op, T::KEY).await?;

        if config.update_complex(value)?.did_execute() {
            self.repo.update_in_op(op, &mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.update_complex", skip(self, value), err)]
    pub async fn update_complex<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        let mut config = self.repo.find_by_key(T::KEY).await?;
        if config.update_complex(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(
        name = "domain_config.upsert_complex_in_op",
        skip(self, op, value),
        err
    )]
    pub async fn upsert_complex_in_op<T>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: T,
    ) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        match self.update_complex_in_op(op, value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create_complex_in_op(op, value).await
            }
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "domain_config.upsert_complex", skip(self, value), err)]
    pub async fn upsert_complex<T>(&self, value: T) -> Result<(), DomainConfigError>
    where
        T: ComplexConfig,
    {
        match self.update_complex(value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create_complex(value).await
            }
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "domain_config.list_simple", skip(self), err)]
    pub async fn list_simple(&self) -> Result<Vec<SimpleEntry>, DomainConfigError> {
        let mut entries = Vec::new();
        let query_args = es_entity::PaginatedQueryArgs::<DomainConfigsByKeyCursor>::default();
        let mut has_next_page = true;
        let mut after = query_args.after;

        let mut op = self.repo.begin_op().await?;
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities: configs,
                has_next_page: next_page,
                end_cursor,
            } = self
                .repo
                .list_by_key_in_op(
                    &mut op,
                    es_entity::PaginatedQueryArgs {
                        first: query_args.first,
                        after,
                    },
                    es_entity::ListDirection::Ascending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            entries.reserve(configs.len());
            for config in configs
                .into_iter()
                .filter(|config| config.config_type.is_simple())
            {
                entries.push(config.into_simple_entry()?);
            }
        }
        op.commit().await?;

        Ok(entries)
    }

    #[instrument(name = "domain_config.get_simple", skip(self), err)]
    pub async fn get_simple<T>(&self) -> Result<T::Scalar, DomainConfigError>
    where
        T: SimpleConfig,
    {
        let config = self.repo.find_by_key(T::KEY).await?;
        config.current_simple_value::<T>()
    }

    #[instrument(name = "domain_config.create_simple", skip(self, value), err)]
    pub async fn create_simple<T>(&self, value: T::Scalar) -> Result<(), DomainConfigError>
    where
        T: SimpleConfig,
    {
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_simple::<T>(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
    }

    #[instrument(name = "domain_config.update_simple", skip(self, value), err)]
    pub async fn update_simple<T>(&self, value: T::Scalar) -> Result<(), DomainConfigError>
    where
        T: SimpleConfig,
    {
        let mut config = self.repo.find_by_key(T::KEY).await?;
        if config.update_simple::<T::Scalar>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[instrument(name = "domain_config.upsert_simple", skip(self, value), err)]
    pub async fn upsert_simple<T>(&self, value: T::Scalar) -> Result<(), DomainConfigError>
    where
        T: SimpleConfig,
    {
        match self.update_simple::<T>(value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create_simple::<T>(value).await
            }
            Err(e) => Err(e),
        }
    }
}
