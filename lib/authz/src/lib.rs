mod all_or_one;
mod check_trait;
#[cfg(feature = "test-dummy")]
pub mod dummy;
pub mod error;

use async_trait::async_trait;
use sqlx_adapter::{
    casbin::{
        prelude::{DefaultModel, Enforcer},
        CoreApi, MgmtApi,
    },
    SqlxAdapter,
};
use std::{fmt, marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;
use tracing::instrument;

use audit::{AuditInfo, AuditSvc};

use error::AuthorizationError;

pub use all_or_one::*;
pub use check_trait::PermissionCheck;

const MODEL: &str = include_str!("./rbac.conf");

#[derive(Clone)]
pub struct Authorization<Audit, R>
where
    Audit: AuditSvc,
    R: Send + Sync + 'static,
{
    enforcer: Arc<RwLock<Enforcer>>,
    audit: Audit,
    _phantom: PhantomData<R>,
}

impl<Audit, R> Authorization<Audit, R>
where
    Audit: AuditSvc,
    R: fmt::Display + fmt::Debug + Clone + Send + Sync,
{
    pub async fn init(pool: &sqlx::PgPool, audit: &Audit) -> Result<Self, AuthorizationError> {
        let model = DefaultModel::from_str(MODEL).await?;
        let adapter = SqlxAdapter::new_with_pool(pool.clone()).await?;

        let enforcer = Enforcer::new(model, adapter).await?;

        let auth = Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
            audit: audit.clone(),
            _phantom: PhantomData,
        };
        Ok(auth)
    }

    pub async fn add_role_hierarchy(
        &self,
        parent_role: R,
        child_role: R,
    ) -> Result<(), AuthorizationError> {
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .add_grouping_policy(vec![child_role.to_string(), parent_role.to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn add_permission_to_role(
        &self,
        role: &R,
        object: impl Into<Audit::Object>,
        action: impl Into<Audit::Action>,
    ) -> Result<(), AuthorizationError> {
        let object = object.into();
        let action = action.into();

        let mut enforcer = self.enforcer.write().await;
        match enforcer
            .add_policy(vec![
                role.to_string(),
                object.to_string(),
                action.to_string(),
            ])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn assign_role_to_subject(
        &self,
        sub: impl Into<Audit::Subject>,
        role: impl std::borrow::Borrow<R>,
    ) -> Result<(), AuthorizationError> {
        let sub = sub.into();
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .add_grouping_policy(vec![sub.to_string(), role.borrow().to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match AuthorizationError::from(e) {
                AuthorizationError::PermissionAlreadyExistsForRole(_) => Ok(()),
                e => Err(e),
            },
        }
    }

    pub async fn revoke_role_from_subject(
        &self,
        sub: impl Into<Audit::Subject>,
        role: impl std::borrow::Borrow<R>,
    ) -> Result<(), AuthorizationError> {
        let sub = sub.into();
        let mut enforcer = self.enforcer.write().await;

        match enforcer
            .remove_grouping_policy(vec![sub.to_string(), role.borrow().to_string()])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(AuthorizationError::from(e)),
        }
    }

    pub async fn roles_for_subject(
        &self,
        sub: impl Into<Audit::Subject>,
    ) -> Result<Vec<R>, AuthorizationError>
    where
        R: std::str::FromStr,
    {
        let sub = sub.into();
        let sub_uuid = sub.to_string();
        let enforcer = self.enforcer.read().await;

        let roles = enforcer
            .get_grouping_policy()
            .into_iter()
            .filter(|r| r[0] == sub_uuid)
            .map(|r| {
                r[1].parse::<R>()
                    .map_err(|_| AuthorizationError::RoleParseError(r[1].clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(roles)
    }

    pub async fn check_all_permissions(
        &self,
        sub: &Audit::Subject,
        object: impl Into<Audit::Object>,
        actions: &[impl Into<Audit::Action> + std::fmt::Debug + Copy],
    ) -> Result<bool, AuthorizationError> {
        let object = object.into();
        for action in actions {
            let action = Into::<Audit::Action>::into(*action);
            match self.enforce_permission(sub, object, action).await {
                Ok(_) => continue,
                Err(AuthorizationError::NotAuthorized) => return Ok(false),
                Err(e) => return Err(e),
            }
        }
        Ok(true)
    }

    async fn inspect_permission(
        &self,
        sub: &Audit::Subject,
        object: impl Into<Audit::Object> + std::fmt::Debug,
        action: impl Into<Audit::Action> + std::fmt::Debug,
    ) -> Result<(), AuthorizationError> {
        let object = object.into();
        let action = action.into();

        let mut enforcer = self.enforcer.write().await;
        enforcer.load_policy().await?;

        match enforcer.enforce((sub.to_string(), object.to_string(), action.to_string())) {
            Ok(true) => Ok(()),
            Ok(false) => Err(AuthorizationError::NotAuthorized),
            Err(e) => Err(AuthorizationError::Casbin(e)),
        }
    }
}

#[async_trait]
impl<Audit, R> PermissionCheck for Authorization<Audit, R>
where
    Audit: AuditSvc,
    R: fmt::Display + fmt::Debug + Clone + Send + Sync + 'static,
{
    type Audit = Audit;

    fn audit(&self) -> &Self::Audit {
        &self.audit
    }

    #[instrument(name = "authz.enforce_permission", skip(self))]
    async fn enforce_permission(
        &self,
        sub: &<Self::Audit as AuditSvc>::Subject,
        object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
    ) -> Result<AuditInfo, AuthorizationError> {
        let object = object.into();
        let action = action.into();

        let result = self.inspect_permission(sub, object, action).await;
        match result {
            Ok(()) => Ok(self.audit.record_entry(sub, object, action, true).await?),
            Err(AuthorizationError::NotAuthorized) => {
                self.audit.record_entry(sub, object, action, false).await?;
                Err(AuthorizationError::NotAuthorized)
            }
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "authz.inspect_permission", skip(self))]
    async fn evaluate_permission(
        &self,
        sub: &<Self::Audit as AuditSvc>::Subject,
        object: impl Into<<Self::Audit as AuditSvc>::Object> + std::fmt::Debug + Send,
        action: impl Into<<Self::Audit as AuditSvc>::Action> + std::fmt::Debug + Send,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, AuthorizationError> {
        let object = object.into();
        let action = action.into();

        if enforce {
            Ok(Some(self.enforce_permission(sub, object, action).await?))
        } else {
            self.inspect_permission(sub, object, action)
                .await
                .map(|_| None)
        }
    }
}
