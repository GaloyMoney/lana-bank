use std::sync::Arc;

use async_graphql::*;

pub use lana_app::notification::NotificationEmailConfig as DomainNotificationEmailConfig;

#[derive(SimpleObject, Clone)]
pub struct NotificationEmailConfig {
    from_email: String,
    from_name: String,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainNotificationEmailConfig>,
}

impl From<DomainNotificationEmailConfig> for NotificationEmailConfig {
    fn from(config: DomainNotificationEmailConfig) -> Self {
        Self {
            from_email: config.from_email.clone(),
            from_name: config.from_name.clone(),
            _entity: Arc::new(config),
        }
    }
}

#[derive(InputObject)]
pub struct NotificationEmailConfigInput {
    pub from_email: String,
    pub from_name: String,
}

impl From<NotificationEmailConfigInput> for DomainNotificationEmailConfig {
    fn from(input: NotificationEmailConfigInput) -> Self {
        Self {
            from_email: input.from_email,
            from_name: input.from_name,
        }
    }
}

crate::mutation_payload! {
    NotificationEmailConfigPayload,
    notification_email_config: NotificationEmailConfig
}
