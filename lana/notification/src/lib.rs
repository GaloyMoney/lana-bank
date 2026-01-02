#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod authorization;
pub mod config;
pub mod email;
pub mod error;

use tracing_macros::record_error_severity;

use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use domain_config::DomainConfigs;
use error::NotificationError;
use job::Jobs;
use lana_events::LanaEvent;

use email::job::{EmailEventListenerConfig, EmailEventListenerInit};
use email::{EmailInfraConfig, EmailNotification, NotificationEmailConfigSpec};

pub use authorization::{
    NotificationAction, NotificationObject, PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_VIEWER,
    PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_WRITER,
};
pub use config::NotificationConfig;
pub use email::NotificationEmailConfig;

pub struct Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    authz: AuthzType,
    domain_configs: DomainConfigs,
    infra_config: EmailInfraConfig,
    email: EmailNotification<AuthzType>,
}

impl<AuthzType> Clone for Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            domain_configs: self.domain_configs.clone(),
            infra_config: self.infra_config.clone(),
            email: self.email.clone(),
        }
    }
}

impl<AuthzType> Notification<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>
        + From<NotificationAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>
        + From<NotificationObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.init", skip_all)]
    pub async fn init(
        config: NotificationConfig,
        jobs: &Jobs,
        outbox: &obix::Outbox<LanaEvent>,
        users: &Users<AuthzType::Audit, LanaEvent>,
        credit: &CoreCredit<AuthzType, LanaEvent>,
        customers: &Customers<AuthzType, LanaEvent>,
        authz: &AuthzType,
        domain_configs: &DomainConfigs,
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

        jobs.add_initializer(EmailEventListenerInit::new(outbox, &email))
            .spawn_unique(::job::JobId::new(), EmailEventListenerConfig::default())
            .await?;

        Ok(Self {
            authz: authz.clone(),
            domain_configs: domain_configs.clone(),
            infra_config: config.email,
            email,
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "notification.get_email_config", skip_all)]
    pub async fn get_email_config(
        &self,
        sub: &<<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject,
    ) -> Result<NotificationEmailConfig, NotificationError> {
        self.authz
            .enforce_permission(
                sub,
                NotificationObject::notification_email_config(),
                NotificationAction::EMAIL_CONFIG_READ,
            )
            .await?;

        let config = self
            .domain_configs
            .get_or_default::<NotificationEmailConfigSpec>()
            .await?;
        Ok(config)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "notification.update_email_config", skip_all)]
    pub async fn update_email_config(
        &self,
        sub: &<<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject,
        new_config: NotificationEmailConfig,
    ) -> Result<NotificationEmailConfig, NotificationError> {
        self.authz
            .enforce_permission(
                sub,
                NotificationObject::notification_email_config(),
                NotificationAction::EMAIL_CONFIG_UPDATE,
            )
            .await?;

        self.domain_configs
            .upsert::<NotificationEmailConfigSpec>(new_config.clone())
            .await?;

        Ok(new_config)
    }
}
