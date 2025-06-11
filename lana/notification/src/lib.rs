pub mod config;
pub mod email;
pub mod error;

pub use config::NotificationConfig;

use authz::Authorization as AuthzAuthorization;
use core_access::user::Users;
use core_credit::CoreCredit;
use core_customer::Customers;
use email::job::{EmailEventListenerConfig, EmailEventListenerInitializer};
use email::EmailNotification;
use job::Jobs;
use lana_events::LanaEvent;
use rbac_types::{LanaAction, LanaObject, Subject};

type LanaAudit = audit::Audit<Subject, LanaObject, LanaAction>;
type Authorization = AuthzAuthorization<LanaAudit, core_access::AuthRoleToken>;
type NotificationOutbox = outbox::Outbox<LanaEvent>;

#[derive(Clone)]
pub struct Notification {
    _email: EmailNotification,
}

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

        Ok(Self { _email: email })
    }
}
