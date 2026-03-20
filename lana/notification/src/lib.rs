#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
pub mod email;
pub mod error;

use tracing_macros::record_error_severity;

use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use domain_config::ExposedDomainConfigsReadOnly;
use error::NotificationError;
use job::Jobs;
use lana_events::LanaEvent;
use obix::out::OutboxEventJobConfig;

pub use config::NotificationConfig;
use email::job::EMAIL_LISTENER_JOB;
pub use email::{NotificationFromEmail, NotificationFromName};

pub struct Notification<Perms>
where
    Perms: authz::PermissionCheck,
{
    _authz: std::marker::PhantomData<Perms>,
}

impl<Perms> Clone for Notification<Perms>
where
    Perms: authz::PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            _authz: std::marker::PhantomData,
        }
    }
}

impl<Perms> Notification<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.init", skip_all)]
    pub async fn init(
        config: NotificationConfig,
        jobs: &mut Jobs,
        outbox: &obix::Outbox<LanaEvent>,
        users: &Users<Perms::Audit, LanaEvent>,
        credit: &CoreCredit<Perms, LanaEvent>,
        customers: &Customers<Perms, LanaEvent>,
        domain_configs: &ExposedDomainConfigsReadOnly,
    ) -> Result<Self, NotificationError> {
        let handler = email::init::<Perms>(
            jobs,
            domain_configs,
            config.email.clone(),
            users,
            credit,
            customers,
        )
        .await?;

        outbox
            .register_event_handler(jobs, OutboxEventJobConfig::new(EMAIL_LISTENER_JOB), handler)
            .await?;

        Ok(Self {
            _authz: std::marker::PhantomData,
        })
    }
}
