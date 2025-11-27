#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use authz::PermissionCheck;
use audit::AuditSvc;
use chrono::Utc;
use es_entity::EntityEvents;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tracing::{Instrument, instrument};

use crate::primitives::DomainConfigurationEntityAction;

pub use entity::*;
pub use error::*;
pub use primitives::*;
pub use repo::*;

#[derive(Clone, Debug)]
pub struct DomainConfigurationRecord<T> {
    pub value: T,
    pub updated_by: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
}

/// Domain-level service to read and write configuration values in a typed manner.
pub struct DomainConfigurations<Perms>
where
    Perms: PermissionCheck,
{
    authz: Perms,
    repo: DomainConfigurationRepo,
}

impl<Perms> Clone for DomainConfigurations<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms> DomainConfigurations<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: ToString,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<DomainConfigurationAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DomainConfigurationObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let repo = DomainConfigurationRepo::new(pool);
        Self {
            authz: authz.clone(),
            repo,
        }
    }

    /// Reads a configuration value for the given key.
    #[instrument(name = "domain_configurations.get", skip(self, sub))]
    pub async fn get<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<T, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: DeserializeOwned,
    {
        self.authz
            .enforce_permission(
                sub,
                K::object(),
                DomainConfigurationAction::new(
                    K::object(),
                    DomainConfigurationEntityAction::Read,
                ),
            )
            .await?;

        let value = self.repo.maybe_find_by_id(K::key()).await?;
        let Some(config) = value else {
            return Err(DomainConfigurationError::NotSet);
        };
        let typed: T = serde_json::from_value(config.value)
            .map_err(|e| DomainConfigurationError::Invalid(e.to_string()))?;
        Ok(typed)
    }

    /// Reads a configuration value along with metadata.
    #[instrument(name = "domain_configurations.get_with_meta", skip(self, sub))]
    pub async fn get_with_meta<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<DomainConfigurationRecord<T>, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: DeserializeOwned,
    {
        self.authz
            .enforce_permission(
                sub,
                K::object(),
                DomainConfigurationAction::new(
                    K::object(),
                    DomainConfigurationEntityAction::Read,
                ),
            )
            .await?;

        let value = self.repo.maybe_find_by_id(K::key()).await?;
        let Some(config) = value else {
            return Err(DomainConfigurationError::NotSet);
        };
        let typed: T = serde_json::from_value(config.value.clone())
            .map_err(|e| DomainConfigurationError::Invalid(e.to_string()))?;

        Ok(DomainConfigurationRecord {
            value: typed,
            updated_by: config.updated_by,
            updated_at: config.updated_at,
            reason: config.reason,
            correlation_id: config.correlation_id,
        })
    }

    /// Writes a configuration value for the given key.
    #[instrument(name = "domain_configurations.set", skip(self, sub, value))]
    pub async fn set<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        value: T,
        reason: Option<String>,
        correlation_id: Option<String>,
    ) -> Result<DomainConfigurationRecord<T>, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: Serialize + DeserializeOwned,
    {
        self.authz
            .enforce_permission(
                sub,
                K::object(),
                DomainConfigurationAction::new(
                    K::object(),
                    DomainConfigurationEntityAction::Write,
                ),
            )
            .await?;

        let now = Utc::now();
        let key = K::key();
        let serialized_value = serde_json::to_value(&value)?;
        let prior = self.repo.maybe_find_by_id(key.clone()).await?;
        let diff = json!({
            "old": prior.as_ref().map(|c| &c.value),
            "new": serialized_value,
        });

        let updated_event = DomainConfigurationEvent::Updated {
            key: key.clone(),
            value: serialized_value.clone(),
            updated_by: sub.to_string(),
            updated_at: now,
            reason: reason.clone(),
            correlation_id: correlation_id.clone(),
            diff,
            previous_value: prior.as_ref().map(|c| c.value.clone()),
        };

        let events = if prior.is_none() {
            EntityEvents::init(key.clone(), [updated_event])
        } else {
            let mut entity = prior.unwrap();
            entity.events.push(updated_event);
            entity.events
        };

        self.repo
            .persist(key, events)
            .instrument(tracing::info_span!("domain_configurations.persist"))
            .await?;

        Ok(DomainConfigurationRecord {
            value,
            updated_by: sub.to_string(),
            updated_at: now,
            reason,
            correlation_id,
        })
    }
}
