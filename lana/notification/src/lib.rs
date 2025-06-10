pub mod config;
pub mod email;
pub mod error;

pub use config::NotificationConfig;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use core_credit::{CoreCredit, CoreCreditAction, CoreCreditObject};
use core_customer::{CoreCustomerAction, CustomerObject, Customers};
use email::job::{EmailEventListenerConfig, EmailEventListenerInitializer};
use email::EmailNotification;
use governance::{GovernanceAction, GovernanceObject};
use job::Jobs;
use lana_events::{
    CoreAccessEvent, CoreCreditEvent, CoreCustomerEvent, GovernanceEvent, LanaEvent,
};
use outbox::OutboxEventMarker;

pub type NotificationOutbox = outbox::Outbox<LanaEvent>;

pub struct Notification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    email: EmailNotification<Perms, E>,
}

impl<Perms, E> Clone for Notification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            email: self.email.clone(),
        }
    }
}

impl<Perms, E> Notification<Perms, E>
where
    Perms: PermissionCheck,
    <Perms::Audit as AuditSvc>::Subject: From<UserId>,
    <Perms::Audit as AuditSvc>::Action: From<CoreAccessAction>
        + From<CoreCreditAction>
        + From<GovernanceAction>
        + From<CoreCustomerAction>,
    <Perms::Audit as AuditSvc>::Object: From<CoreAccessObject>
        + From<CoreCreditObject>
        + From<GovernanceObject>
        + From<CustomerObject>,
    E: OutboxEventMarker<CoreAccessEvent>
        + OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        jobs: &Jobs,
        outbox: &NotificationOutbox,
        config: NotificationConfig,
        users: &Users<Perms::Audit, E>,
        credit: &CoreCredit<Perms, E>,
        customers: &Customers<Perms, E>,
    ) -> Result<Self, error::NotificationError> {
        let email = EmailNotification::init(jobs, config.email, users, credit, customers).await?;
        jobs.add_initializer_and_spawn_unique(
            EmailEventListenerInitializer::new(outbox, &email),
            EmailEventListenerConfig::new(),
        )
        .await?;

        Ok(Self { email })
    }
}
