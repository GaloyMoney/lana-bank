use std::{fmt::Display, str::FromStr};

use authz::{ActionPermission, action_description::*, map_action};

pub const PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_VIEWER: &str = "notification_email_viewer";
pub const PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_WRITER: &str = "notification_email_writer";

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum NotificationObject {
    NotificationEmailConfig,
}

impl NotificationObject {
    pub const fn notification_email_config() -> Self {
        Self::NotificationEmailConfig
    }
}

impl Display for NotificationObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let discriminant = NotificationObjectDiscriminants::from(self);
        write!(f, "{discriminant}/*")
    }
}

impl FromStr for NotificationObject {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (entity, _id) = s.split_once('/').expect("missing slash");

        use NotificationObjectDiscriminants::*;
        let res = match entity.parse().expect("invalid entity") {
            NotificationEmailConfig => NotificationObject::NotificationEmailConfig,
        };

        Ok(res)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::Display, strum::EnumString, strum::VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub enum NotificationEmailConfigAction {
    Read,
    Update,
}

impl ActionPermission for NotificationEmailConfigAction {
    fn permission_set(&self) -> &'static str {
        match self {
            Self::Read => PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_VIEWER,
            Self::Update => PERMISSION_SET_NOTIFICATION_EMAIL_CONFIG_WRITER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::EnumString, strum::VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum NotificationAction {
    NotificationEmailConfig(NotificationEmailConfigAction),
}

impl NotificationAction {
    pub const EMAIL_CONFIG_READ: Self =
        Self::NotificationEmailConfig(NotificationEmailConfigAction::Read);
    pub const EMAIL_CONFIG_UPDATE: Self =
        Self::NotificationEmailConfig(NotificationEmailConfigAction::Update);

    pub fn actions() -> Vec<ActionMapping> {
        use NotificationActionDiscriminants::*;

        map_action!(
            notification,
            NotificationEmailConfig,
            NotificationEmailConfigAction
        )
    }
}

impl From<NotificationEmailConfigAction> for NotificationAction {
    fn from(action: NotificationEmailConfigAction) -> Self {
        NotificationAction::NotificationEmailConfig(action)
    }
}

impl Display for NotificationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", NotificationActionDiscriminants::from(self))?;
        match self {
            NotificationAction::NotificationEmailConfig(action) => action.fmt(f),
        }
    }
}

impl FromStr for NotificationAction {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (module, action) = s.split_once(':').expect("missing colon");
        use NotificationActionDiscriminants::*;
        let res = match module.parse()? {
            NotificationEmailConfig => {
                NotificationAction::from(action.parse::<NotificationEmailConfigAction>()?)
            }
        };
        Ok(res)
    }
}
