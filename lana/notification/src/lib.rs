#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod email;
pub mod error;

use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use job::Jobs;
use lana_events::LanaEvent;

use email::EmailNotification;
use email::job::{EmailEventListenerConfig, EmailEventListenerInit};

pub use config::NotificationConfig;

pub struct Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    _authz: std::marker::PhantomData<AuthzType>,
}

impl<AuthzType> Clone for Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            _authz: std::marker::PhantomData,
        }
    }
}

impl<AuthzType> Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_credit::CoreCreditAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_customer::CoreCustomerAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_access::CoreAccessAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<governance::GovernanceAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_credit::CoreCreditObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_customer::CustomerObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_access::CoreAccessObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<governance::GovernanceObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub async fn init(
        config: NotificationConfig,
        jobs: &Jobs,
        outbox: &outbox::Outbox<LanaEvent>,
        users: &Users<AuthzType::Audit, LanaEvent>,
        credit: &CoreCredit<AuthzType, LanaEvent>,
        customers: &Customers<AuthzType, LanaEvent>,
    ) -> Result<Self, error::NotificationError> {
        let email = EmailNotification::init(jobs, config.email, users, credit, customers).await?;
        jobs.add_initializer_and_spawn_unique(
            EmailEventListenerInit::new(outbox, &email),
            EmailEventListenerConfig::default(),
        )
        .await?;

        Ok(Self {
            _authz: std::marker::PhantomData,
        })
    }
}
