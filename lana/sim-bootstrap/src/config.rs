use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BootstrapConfig {
    #[serde(default = "default_num_facilities")]
    pub num_facilities: u32,
    #[serde(default = "default_num_customers")]
    pub num_customers: u32,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            num_facilities: default_num_facilities(),
            num_customers: default_num_customers(),
        }
    }
}

fn default_num_facilities() -> u32 {
    1
}

fn default_num_customers() -> u32 {
    1
}
