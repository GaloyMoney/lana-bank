//! # Domain Config
//!
//! Type-safe, event-sourced configuration management for domain-specific settings.
//!
//! ## Quick Start
//!
//! Define a configuration using one of the two macros:
//!
//! ### Exposed Configs (modifiable via API/UI)
//!
//! ```ignore
//! use domain_config::{define_exposed_config, DomainConfigError};
//!
//! define_exposed_config! {
//!     pub struct MyEmailConfig(String);
//!     spec {
//!         key: "my-email-config";
//!         validate: |value: &String| {
//!             if value.is_empty() {
//!                 return Err(DomainConfigError::InvalidState("cannot be empty".into()));
//!             }
//!             Ok(())
//!         };
//!     }
//! }
//! ```
//!
//! ### Internal Configs (programmatic access only)
//!
//! For simple values:
//! ```ignore
//! use domain_config::define_internal_config;
//!
//! define_internal_config! {
//!     pub struct MaxRetries(u64);
//!     spec {
//!         key: "max-retries";
//!         default: || Some(3);
//!     }
//! }
//! ```
//!
//! For complex structs (must derive `Serialize` + `Deserialize`):
//! ```ignore
//! use domain_config::define_internal_config;
//! use serde::{Serialize, Deserialize};
//!
//! define_internal_config! {
//!     #[derive(Clone, Debug, Serialize, Deserialize)]
//!     pub struct FeatureFlags {
//!         pub enable_notifications: bool,
//!         pub max_batch_size: u64,
//!     }
//!     spec {
//!         key: "feature-flags";
//!     }
//! }
//! ```
//!
//! ## Reading and Writing Configs
//!
//! ```ignore
//! let configs = DomainConfigs::new(&pool);
//!
//! // Create initial value
//! configs.create::<MaxRetries>(5).await?;
//!
//! // Update existing value
//! configs.update::<MaxRetries>(10).await?;
//!
//! // Create or update (upsert)
//! configs.upsert::<MaxRetries>(10).await?;
//!
//! // Read config (returns default if not set but default exists)
//! let config = configs.get::<MaxRetries>().await?;
//! let value: Option<u64> = config.value();
//! ```
//!
//! ## Spec Options
//!
//! Both macros support these optional spec fields:
//! - `key`: (required) Unique string identifier for the config
//! - `default`: Optional closure returning `Option<T>` for default value
//! - `validate`: Optional closure for validation, returns `Result<(), DomainConfigError>`
//!
//! ## Visibility Difference
//!
//! | Macro | Visibility | Use Case |
//! |-------|------------|----------|
//! | `define_exposed_config!` | `Exposed` | Settings modifiable by users via API/UI |
//! | `define_internal_config!` | `Internal` | System settings, only changed programmatically |
//!
//! ## Supported Simple Types
//!
//! For tuple-struct configs: `bool`, `String`, `i64`, `u64`, `rust_decimal::Decimal`
//!
//! For complex configs (struct form): Any type implementing `Serialize + DeserializeOwned`

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
pub struct DomainConfigs {
    repo: DomainConfigRepo,
}

impl DomainConfigs {
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
        let mut seen = HashSet::new();
        for spec in registry::all_specs() {
            if !seen.insert(spec.key) {
                return Err(DomainConfigError::InvalidKey(format!(
                    "Duplicate domain config key: {}",
                    spec.key
                )));
            }

            let key = DomainConfigKey::new(spec.key);
            let config_id = DomainConfigId::new();
            let new = NewDomainConfig::builder()
                .seed(config_id, key, spec.config_type, spec.visibility)
                .build()?;
            match self.repo.create(new).await {
                Ok(_) => {}
                Err(DomainConfigError::DuplicateKey) => continue,
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
