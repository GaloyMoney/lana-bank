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
use email::EmailNotification;
use email::job::{
    EMAIL_LISTENER_JOB, EmailEventListenerHandler, ProcessEmailNotificationJobInitializer,
};
pub use email::{NotificationFromEmail, NotificationFromName};

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
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit::CoreCreditCollateralAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit::CoreCreditCollateralObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.init", skip_all)]
    pub async fn init(
        config: NotificationConfig,
        jobs: &mut Jobs,
        outbox: &obix::Outbox<LanaEvent>,
        users: &Users<AuthzType::Audit, LanaEvent>,
        credit: &CoreCredit<AuthzType, LanaEvent>,
        customers: &Customers<AuthzType, LanaEvent>,
        domain_configs: &ExposedDomainConfigsReadOnly,
    ) -> Result<Self, NotificationError> {
        let email = EmailNotification::init(
            jobs,
            domain_configs,
            config.email.clone(),
            users,
            credit,
            customers,
        )
        .await?;

        let process_email_notification =
            jobs.add_initializer(ProcessEmailNotificationJobInitializer::new(email));
        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(EMAIL_LISTENER_JOB),
                EmailEventListenerHandler::new(process_email_notification),
            )
            .await?;

        Ok(Self {
            _authz: std::marker::PhantomData,
        })
    }
}
