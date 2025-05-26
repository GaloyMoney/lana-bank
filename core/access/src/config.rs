use serde::{Deserialize, Serialize};

use authz::action_description::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    pub superuser_email: Option<String>,
    #[serde(skip)]
    pub action_descriptions: Vec<ActionDescription<FullPath>>,
    #[serde(skip)]
    pub predefined_roles: &'static [(&'static str, &'static [&'static str])],
}
