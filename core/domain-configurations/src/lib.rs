#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod publisher;
mod repo;

pub use entity::{DomainConfiguration, DomainConfigurationEvent, DomainConfigurationRecord};
pub use error::DomainConfigurationError;
pub use primitives::*;
pub use publisher::DomainConfigurationPublisher;

use crate::entity::NewDomainConfiguration;
use authz::PermissionCheck;
use chrono::Utc;
use outbox::OutboxEventMarker;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::instrument;

use publisher::DomainConfigurationPublisher;
use repo::DomainConfigurationRepo;

pub struct DomainConfigurations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    repo: DomainConfigurationRepo<E>,
    authz: Perms,
}

impl<Perms, E> Clone for DomainConfigurations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<DomainConfigurationEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms, E> DomainConfigurations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<DomainConfigurationEvent>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<DomainConfigurationAction>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<DomainConfigurationObject>,
{
    pub fn new(
        pool: &PgPool,
        authz: &Perms,
        publisher: &DomainConfigurationPublisher<E>,
    ) -> Self {
        let repo = DomainConfigurationRepo::new(pool, publisher);
        Self {
            repo,
            authz: authz.clone(),
        }
    }

    #[instrument(name = "domain_configurations.get", skip(self, sub), err)]
    pub async fn get<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Subject,
    ) -> Result<T, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: DeserializeOwned,
    {
        let record = self.get_with_meta::<K, T>(sub).await?;
        Ok(record.value)
    }

    #[instrument(name = "domain_configurations.get_with_meta", skip(self, sub), err)]
    pub async fn get_with_meta<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Subject,
    ) -> Result<DomainConfigurationRecord<T>, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: DeserializeOwned,
    {
        self.authz
            .enforce_permission(sub, K::object(), K::required_action_read())
            .await
            .map_err(DomainConfigurationError::from)?;

        let Some(config) = self.repo.maybe_find_by_id(K::key()).await? else {
            return Err(DomainConfigurationError::NotSet);
        };

        let value = serde_json::from_value(config.value.clone())
            .map_err(|e| DomainConfigurationError::Invalid(e.to_string()))?;

        Ok(DomainConfigurationRecord {
            value,
            updated_by: config.updated_by,
            updated_at: config.updated_at,
            reason: config.reason,
            correlation_id: config.correlation_id,
        })
    }

    #[instrument(name = "domain_configurations.set", skip(self, sub), err)]
    pub async fn set<K, T>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Subject,
        value: T,
        reason: Option<String>,
        correlation_id: Option<String>,
    ) -> Result<DomainConfigurationRecord<T>, DomainConfigurationError>
    where
        K: ConfigKey<T>,
        T: Serialize + Clone + DeserializeOwned,
    {
        let audit_info = self
            .authz
            .enforce_permission(sub, K::object(), K::required_action_write())
            .await
            .map_err(DomainConfigurationError::from)?;

        let value_json = serde_json::to_value(value.clone())
            .map_err(|e| DomainConfigurationError::Invalid(e.to_string()))?;

        let mut op = self.repo.begin_op().await?;

        let updated_at = Utc::now();

        let config = match self.repo.maybe_find_by_id(K::key()).await? {
            Some(mut existing) => {
                existing.apply_update(
                    audit_info,
                    updated_at,
                    value_json.clone(),
                    reason.clone(),
                    correlation_id.clone(),
                );
                self.repo.update_in_op(&mut op, &mut existing).await?;
                existing
            }
            None => {
                let new = entity::NewDomainConfiguration::new(
                    K::key(),
                    audit_info,
                    updated_at,
                    value_json.clone(),
                    reason.clone(),
                    correlation_id.clone(),
                );
                self.repo.create_in_op(&mut op, new).await?
            }
        };

        Ok(DomainConfigurationRecord {
            value,
            updated_by: config.updated_by,
            updated_at: config.updated_at,
            reason: config.reason,
            correlation_id: config.correlation_id,
        })
    }
}
