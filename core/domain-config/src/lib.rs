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

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entity::DomainConfig;
pub use entity::DomainConfigEvent;
pub use error::DomainConfigError;
#[doc(hidden)]
pub use inventory;
pub use primitives::{
    ConfigType, DomainConfigAction, DomainConfigEntityAction, DomainConfigId, DomainConfigKey,
    DomainConfigObject, ExposedConfigAllOrOne, PERMISSION_SET_EXPOSED_CONFIGS_VIEWER,
    PERMISSION_SET_EXPOSED_CONFIGS_WRITER, Visibility,
};
pub use repo::domain_config_cursor::DomainConfigsByKeyCursor;
pub use spec::{Complex, ConfigSpec, ExposedConfig, InternalConfig, Simple, ValueKind};
pub use typed_domain_config::TypedDomainConfig;

use entity::NewDomainConfig;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DomainConfigEvent;
}

use repo::DomainConfigRepo;

#[derive(Clone)]
pub struct InternalDomainConfigs {
    repo: DomainConfigRepo,
}

#[derive(Clone)]
pub struct ExposedDomainConfigs<Perms>
where
    Perms: PermissionCheck,
{
    repo: DomainConfigRepo,
    authz: Perms,
}

impl InternalDomainConfigs {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self { repo }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self))]
    pub async fn get<C>(&self) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: InternalConfig,
    {
        let config = self.repo.find_by_key(C::KEY).await?;
        TypedDomainConfig::new(config)
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.create", skip(self, value))]
    pub async fn create<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: InternalConfig,
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
        C: InternalConfig,
    {
        let mut config = self.repo.find_by_key(C::KEY).await?;
        if config.update_value::<C>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.upsert", skip(self, value))]
    pub async fn upsert<C>(
        &self,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: InternalConfig,
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
    #[instrument(name = "domain_config.seed_registered", skip(self))]
    pub async fn seed_registered(&self) -> Result<(), DomainConfigError> {
        seed_registered_for_visibility(&self.repo, Visibility::Internal).await
    }
}

impl<Perms> ExposedDomainConfigs<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DomainConfigObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let repo = DomainConfigRepo::new(pool);
        Self {
            repo,
            authz: authz.clone(),
        }
    }

    async fn ensure_exposed_read(
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

    async fn ensure_exposed_write(
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

    #[record_error_severity]
    #[instrument(name = "domain_config.get", skip(self), fields(subject = %sub))]
    pub async fn get<C>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<TypedDomainConfig<C>, DomainConfigError>
    where
        C: ExposedConfig,
    {
        self.ensure_exposed_read(sub).await?;
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
    #[instrument(name = "domain_config.create", skip(self, value), fields(subject = %sub))]
    pub async fn create<C>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ExposedConfig,
    {
        self.ensure_exposed_write(sub).await?;
        let domain_config_id = DomainConfigId::new();
        let new = NewDomainConfig::builder()
            .with_value::<C>(domain_config_id, value)?
            .build()
            .expect("Could not build NewDomainConfig");
        self.repo.create(new).await?;

        Ok(())
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
    {
        self.ensure_exposed_write(sub).await?;
        let mut config = self.repo.find_by_key(C::KEY).await?;
        if config.update_value::<C>(value)?.did_execute() {
            self.repo.update(&mut config).await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.update_exposed_from_json", skip(self, value), fields(subject = %sub))]
    pub async fn update_exposed_from_json(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DomainConfigId> + std::fmt::Debug,
        value: serde_json::Value,
    ) -> Result<DomainConfig, DomainConfigError> {
        self.ensure_exposed_write(sub).await?;
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
    #[instrument(name = "domain_config.upsert", skip(self, value), fields(subject = %sub))]
    pub async fn upsert<C>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        value: <C::Kind as ValueKind>::Value,
    ) -> Result<(), DomainConfigError>
    where
        C: ExposedConfig,
        <C::Kind as ValueKind>::Value: Clone,
    {
        match self.update::<C>(sub, value.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainConfigError::EsEntityError(es_entity::EsEntityError::NotFound)) => {
                self.create::<C>(sub, value).await
            }
            Err(e) => Err(e),
        }
    }

    #[record_error_severity]
    #[instrument(name = "domain_config.list_exposed_configs", skip(self, query), fields(subject = %sub))]
    pub async fn list_exposed_configs(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<DomainConfigsByKeyCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DomainConfig, DomainConfigsByKeyCursor>,
        DomainConfigError,
    > {
        self.ensure_exposed_read(sub).await?;
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
        seed_registered_for_visibility(&self.repo, Visibility::Exposed).await
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
            .seed(config_id, key, spec.config_type, spec.visibility)
            .build()?;
        match repo.create(new).await {
            Ok(_) => {}
            Err(DomainConfigError::DuplicateKey) => continue,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
