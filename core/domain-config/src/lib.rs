#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

//! # Domain Config
//!
//! Type-safe, persistent configuration storage with two visibility levels.
//!
//! Supported simple types: `bool`, `i64`, `u64`, `String`, `Decimal`, `Timezone`, `Time`.
//! Internal configs also support complex structs. Exposed configs only support simple types.
//!
//! ## Defining Configs
//!
//! **Internal** configs are owned by another crate which defines custom authorization:
//! ```
//! domain_config::define_internal_config! {
//!     pub struct MyInternalFlag(bool);
//!     spec {
//!         key: "my-internal-flag";
//!     }
//! }
//! ```
//!
//! Complex structs are also supported for internal configs:
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! domain_config::define_internal_config! {
//!     #[derive(Clone, Debug, Serialize, Deserialize)]
//!     pub struct MyComplexConfig {
//!         pub limit: u32,
//!     }
//!     spec {
//!         key: "my-complex-config";
//!     }
//! }
//! ```
//!
//! **Exposed** configs use standard domain-config authorization and automatically
//! appear in the admin app's Configurations page:
//! ```
//! domain_config::define_exposed_config! {
//!     pub struct MyExposedSetting(String);
//!     spec {
//!         key: "my-exposed-setting";
//!     }
//! }
//! ```
//!
//! Both macros support optional `default:` and `validate:` clauses:
//! ```
//! domain_config::define_internal_config! {
//!     pub struct MaxRetries(u64);
//!     spec {
//!         key: "max-retries";
//!         default: || Some(3);
//!         validate: |v: &u64| if *v > 0 && *v <= 100 {
//!             Ok(())
//!         } else {
//!             Err(domain_config::DomainConfigError::InvalidState(
//!                 "max-retries must be between 1 and 100".into()
//!             ))
//!         };
//!     }
//! }
//! ```
//!
//! ## Using Configs
//!
//! Use [`InternalDomainConfigs`] for internal configs. The owning crate is responsible
//! for enforcing authorization before calling these methods:
//! ```no_run
//! # domain_config::define_internal_config! {
//! #     pub struct MyInternalFlag(bool);
//! #     spec {
//! #         key: "my-internal-flag";
//! #     }
//! # }
//! # async fn example(pool: &sqlx::PgPool) -> Result<(), domain_config::DomainConfigError> {
//!     // Enforce your custom authorization here before accessing the config
//!     let configs = domain_config::InternalDomainConfigs::new(pool);
//!     let value = configs.get::<MyInternalFlag>().await?.maybe_value();
//!     configs.update::<MyInternalFlag>(true).await?;
//! #     Ok(())
//! # }
//! ```
//!
//! Use [`ExposedDomainConfigs`] for exposed configs (requires auth subject):
//! ```no_run
//! # domain_config::define_exposed_config! {
//! #     pub struct MyExposedSetting(String);
//! #     spec {
//! #         key: "my-exposed-setting";
//! #     }
//! # }
//! # async fn example<P: authz::PermissionCheck>(
//! #     pool: &sqlx::PgPool,
//! #     authz: &P,
//! #     subject: &<P::Audit as audit::AuditSvc>::Subject,
//! # ) -> Result<(), domain_config::DomainConfigError>
//! # where
//! #     <<P as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
//! #         From<domain_config::DomainConfigAction>,
//! #     <<P as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
//! #         From<domain_config::DomainConfigObject>,
//! # {
//!     let configs = domain_config::ExposedDomainConfigs::new(pool, authz);
//!     let value = configs.get::<MyExposedSetting>(subject).await?.maybe_value();
//!     configs.update::<MyExposedSetting>(subject, "new-value".into()).await?;
//! #     Ok(())
//! # }
//! ```
//!
//! Use [`ExposedDomainConfigsReadOnly`] for read-only access without authorization
//! (for background jobs and internal processes):
//! ```no_run
//! # domain_config::define_exposed_config! {
//! #     pub struct MyExposedSetting(String);
//! #     spec {
//! #         key: "my-exposed-setting";
//! #     }
//! # }
//! # async fn example(pool: &sqlx::PgPool) -> Result<(), domain_config::DomainConfigError> {
//!     let configs = domain_config::ExposedDomainConfigsReadOnly::new(pool);
//!     let value = configs.get_without_audit::<MyExposedSetting>().await?.maybe_value();
//! #     Ok(())
//! # }
//! ```
//!
//! ## Config Lifecycle
//!
//! Using the `define_internal_config!` or `define_exposed_config!` macros automatically
//! registers your config for seeding. At application startup, all registered configs are
//! seeded to the database. Developers defining new configs do not need to call any seeding
//! functions manually - just use the macro and the config will be available. Because of
//! this automatic seeding, `get` always succeeds for all configs.
//!
//! ## Accessing Config Values
//!
//! For configs **with defaults** (defined with a `default:` clause), use `value()`:
//! - Returns `T` directly (not `Option<T>`)
//! - Always succeeds because the default guarantees a value
//!
//! For configs **without defaults**, use `maybe_value()`:
//! - Returns `Option<T>`
//! - `Some(value)` if the config has been explicitly set via `update`
//! - `None` if no value is set

mod config;
mod encryption;
mod entity;
pub mod error;
mod flavor;
mod macros;
mod primitives;
pub mod registry;
mod repo;
mod shared_config;
mod spec;
mod typed_domain_config;
mod value;

use std::collections::{HashMap, HashSet};

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use config::EncryptionConfig;
pub use entity::DomainConfig;
pub use entity::DomainConfigEvent;
pub use error::DomainConfigError;
pub use flavor::FlavorDispatch;
#[doc(hidden)]
pub use inventory;
pub use primitives::{
    ConfigType, DomainConfigAction, DomainConfigEntityAction, DomainConfigId, DomainConfigKey,
    DomainConfigObject, ExposedConfigAllOrOne, PERMISSION_SET_EXPOSED_CONFIG_VIEWER,
    PERMISSION_SET_EXPOSED_CONFIG_WRITER, Visibility,
};
pub use repo::domain_config_cursor::DomainConfigsByKeyCursor;
pub use shared_config::RequireVerifiedCustomerForAccount;
pub use spec::{
    Complex, ConfigFlavor, ConfigSpec, DefaultedConfig, DomainConfigFlavorEncrypted,
    DomainConfigFlavorPlaintext, ExposedConfig, InternalConfig, Simple, ValueKind,
};
pub use typed_domain_config::TypedDomainConfig;
pub use value::DomainConfigValue;

use entity::NewDomainConfig;
use repo::DomainConfigRepo;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DomainConfigEvent;
}

#[derive(Clone)]
pub struct InternalDomainConfigs {
    repo: DomainConfigRepo,
    config: EncryptionConfig,
}

#[derive(Clone)]
pub struct ExposedDomainConfigs<Perms>
where
    Perms: PermissionCheck,
{
    repo: DomainConfigRepo,
    authz: Perms,
    config: EncryptionConfig,
}

/// Read-only access to exposed domain configs without authorization.
///
/// Use for internal consumers (jobs, background processes) that need
/// to read exposed config values without user context.
#[derive(Clone)]
pub struct ExposedDomainConfigsReadOnly {
    repo: DomainConfigRepo,
    config: EncryptionConfig,
}

impl InternalDomainConfigs {
    pub fn new(pool: &sqlx::PgPool, config: EncryptionConfig) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo, config }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self))]
    pub async fn get<C>(&self) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: InternalConfig,
        C::Flavor: FlavorDispatch,
    {
        let entity = self.repo.find_by_key(C::KEY).await?;
        C::Flavor::try_new::<C>(entity, &self.config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update", skip(self, value))]
    pub async fn update<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: InternalConfig,
        C::Flavor: FlavorDispatch,
    {
        let mut entity = self.repo.find_by_key(C::KEY).await?;
        if C::Flavor::update_value::<C>(&mut entity, &self.config, value)?.did_execute() {
            self.repo.update(&mut entity).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.seed_registered", skip(self))]
    pub async fn seed_registered(&self) -> Result<(), DomainConfigError> {
        seed_registered_for_visibility(&self.repo, Visibility::Internal).await
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.begin_op", skip(self))]
    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'static>, DomainConfigError> {
        Ok(self.repo.begin_op().await?)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update_in_op", skip(self, op, value))]
    pub async fn update_in_op<C>(
        &self,
        op: &mut es_entity::DbOp<'_>,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: InternalConfig,
        C::Flavor: FlavorDispatch,
    {
        let mut entity = self.repo.find_by_key_in_op(&mut *op, C::KEY).await?;
        if C::Flavor::update_value::<C>(&mut entity, &self.config, value)?.did_execute() {
            self.repo.update_in_op(op, &mut entity).await?;
        }
        Ok(())
    }
}

impl ExposedDomainConfigsReadOnly {
    pub fn new(pool: &sqlx::PgPool, config: EncryptionConfig) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo, config }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get_without_audit", skip(self))]
    pub async fn get_without_audit<C>(&self) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: ExposedConfig,
        C::Flavor: FlavorDispatch,
    {
        let entity = self.repo.find_by_key(C::KEY).await?;
        C::Flavor::try_new::<C>(entity, &self.config)
    }
}

impl<Perms> ExposedDomainConfigs<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DomainConfigObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, config: EncryptionConfig) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self {
            repo,
            authz: authz.clone(),
            config,
        }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self), fields(subject = %sub))]
    pub async fn get<C>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: ExposedConfig,
        C::Flavor: FlavorDispatch,
    {
        self.ensure_read_permission(sub).await?;
        let entity = self.repo.find_by_key(C::KEY).await?;
        C::Flavor::try_new::<C>(entity, &self.config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update", skip(self, value), fields(subject = %sub))]
    pub async fn update<C>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ExposedConfig,
        C::Flavor: FlavorDispatch,
    {
        self.ensure_write_permission(sub).await?;
        let mut entity = self.repo.find_by_key(C::KEY).await?;

        if C::Flavor::update_value::<C>(&mut entity, &self.config, value)?.did_execute() {
            self.repo.update(&mut entity).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.list_exposed_configs", skip(self, query), fields(subject = %sub))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<DomainConfigsByKeyCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DomainConfig, DomainConfigsByKeyCursor>,
        DomainConfigError,
    > {
        self.ensure_read_permission(sub).await?;
        self.repo
            .list_for_visibility_by_key(
                Visibility::Exposed,
                query,
                es_entity::ListDirection::Ascending,
            )
            .await
    }

    /// This is a GraphQL batch loader helper (no subject parameter), so it mirrors our common
    /// pattern where DataLoader “find_all” methods are auth-free and are only called after a
    /// higher‑level, subject‑aware endpoint has already enforced access. In practice it’s used for
    /// GraphQL field loading, which doesn’t carry subject into the loader.
    #[record_error_severity]
    #[instrument(name = "domain_config.find_all_exposed", skip(self))]
    pub async fn find_all<T: From<DomainConfig>>(
        &self,
        ids: &[DomainConfigId],
    ) -> Result<HashMap<DomainConfigId, T>, DomainConfigError> {
        self.repo.find_all_exposed(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update_exposed_from_json", skip(self, value), fields(subject = %sub))]
    pub async fn update_from_json(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DomainConfigId> + std::fmt::Debug,
        value: serde_json::Value,
    ) -> Result<DomainConfig, DomainConfigError> {
        self.ensure_write_permission(sub).await?;
        let id = id.into();
        let mut entity = self.repo.find_by_id(id).await?;
        let entry = registry::maybe_find_by_key(entity.key.as_str()).ok_or_else(|| {
            DomainConfigError::InvalidKey(format!(
                "Registry entry missing for config key: {}",
                entity.key
            ))
        })?;

        if entity
            .apply_exposed_update_from_json(entry, &self.config, value)?
            .did_execute()
        {
            self.repo.update(&mut entity).await?;
        }

        Ok(entity)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.seed_registered", skip(self))]
    pub async fn seed_registered(&self) -> Result<(), DomainConfigError> {
        seed_registered_for_visibility(&self.repo, Visibility::Exposed).await
    }

    async fn ensure_read_permission(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<(), DomainConfigError> {
        self.authz
            .enforce_permission(
                sub,
                DomainConfigObject::all_exposed_configs(),
                DomainConfigAction::EXPOSED_CONFIG_READ,
            )
            .await?;
        Ok(())
    }

    async fn ensure_write_permission(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<(), DomainConfigError> {
        self.authz
            .enforce_permission(
                sub,
                DomainConfigObject::all_exposed_configs(),
                DomainConfigAction::EXPOSED_CONFIG_WRITE,
            )
            .await?;
        Ok(())
    }
}

async fn seed_registered_for_visibility(
    repo: &DomainConfigRepo,
    visibility: Visibility,
) -> Result<(), DomainConfigError> {
    let mut seen = HashSet::new();
    for spec in registry::all_specs() {
        if !seen.insert(spec.key) {
            return Err(DomainConfigError::InvalidKey(format!(
                "Duplicate domain config key: {}",
                spec.key
            )));
        }

        if spec.visibility != visibility {
            continue;
        }

        let key = DomainConfigKey::new(spec.key);
        let config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .seed(
                config_id,
                key,
                spec.config_type,
                spec.visibility,
                spec.encrypted,
            )
            .build()?;
        match repo.create(new).await {
            Ok(_) => {}
            Err(DomainConfigError::DuplicateKey) => continue,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

/// Apply domain config settings at startup from key-value pairs.
///
/// This is intended for bootstrap scenarios where configs need to be set
/// from environment variables before user context is available.
/// Works for both internal and exposed configs.
///
/// All settings are applied atomically within a single transaction.
#[instrument(name = "domain_config.apply_startup_configs", skip_all)]
pub async fn apply_startup_configs<I, K>(
    pool: &sqlx::PgPool,
    encryption_config: &EncryptionConfig,
    settings: I,
) -> Result<(), DomainConfigError>
where
    I: IntoIterator<Item = (K, serde_json::Value)>,
    K: Into<DomainConfigKey> + std::fmt::Display + Clone,
{
    let repo = DomainConfigRepo::new(pool);
    let mut db_tx = repo.begin_op().await?;

    for (key, value) in settings {
        let domain_key: DomainConfigKey = key.clone().into();
        let entry = match registry::maybe_find_by_key(domain_key.as_str()) {
            Some(entry) => entry,
            None => {
                tracing::error!(key = %key, "Unknown domain config key, skipping");
                continue;
            }
        };

        let mut entity = repo.find_by_key_in_op(&mut db_tx, domain_key).await?;

        let changed = entity
            .apply_update_from_json(entry, encryption_config, value)?
            .did_execute();

        if changed {
            repo.update_in_op(&mut db_tx, &mut entity).await?;
            tracing::info!(key = %key, "Applied domain config at startup");
        } else {
            tracing::info!(key = %key, "Domain config already set");
        }
    }

    db_tx.commit().await?;
    Ok(())
}
