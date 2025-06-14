pub mod config;
pub mod email;
pub mod error;

use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use job::Jobs;
use lana_events::LanaEvent;
use rbac_types::{LanaAction, LanaObject, Subject};

use email::EmailNotification;
use email::job::{EmailEventListenerConfig, EmailEventListenerInitializer};

pub use config::NotificationConfig;

pub(crate) type LanaAudit = audit::Audit<Subject, LanaObject, LanaAction>;
pub(crate) type Authorization = authz::Authorization<LanaAudit, core_access::AuthRoleToken>;
pub(crate) type NotificationOutbox = outbox::Outbox<LanaEvent>;

pub struct Notification;

impl Notification {
    pub async fn init(
        config: NotificationConfig,
        jobs: &Jobs,
        outbox: &NotificationOutbox,
        users: &Users<LanaAudit, LanaEvent>,
        credit: &CoreCredit<Authorization, LanaEvent>,
        customers: &Customers<Authorization, LanaEvent>,
    ) -> Result<Self, error::NotificationError> {
        let email = EmailNotification::init(jobs, config.email, users, credit, customers).await?;
        jobs.add_initializer_and_spawn_unique(
            EmailEventListenerInitializer::new(outbox, &email),
            EmailEventListenerConfig,
        )
        .await?;

        Ok(Self)
    }
}
