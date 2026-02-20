use crate::{ConfigType, Visibility};

#[derive(Debug, Clone, Copy)]
pub struct ConfigSpecEntry {
    pub key: &'static str,
    pub visibility: Visibility,
    pub config_type: ConfigType,
    pub encrypted: bool,
    pub validate_json: fn(&serde_json::Value) -> Result<(), crate::DomainConfigError>,
}

#[linkme::distributed_slice]
pub static CONFIG_SPECS: [ConfigSpecEntry];

pub fn all_specs() -> impl Iterator<Item = &'static ConfigSpecEntry> {
    CONFIG_SPECS.iter()
}

pub fn maybe_find_by_key(key: &str) -> Option<&'static ConfigSpecEntry> {
    all_specs().find(|spec| spec.key == key)
}
