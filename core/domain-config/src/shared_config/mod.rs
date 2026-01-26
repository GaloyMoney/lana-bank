//! Shared exposed configs used by multiple core modules.
//!
//! Configs are defined here to avoid circular dependencies between modules
//! (e.g., credit depending on deposit or vice versa).

use crate::define_exposed_config;

define_exposed_config! {
    /// Controls whether KYC verification is required for account operations.
    /// When enabled, customers must be KYC-verified before creating
    /// deposit accounts or credit facilities.
    pub struct RequireVerifiedCustomerForAccount(bool);
    spec {
        key: "require-verified-customer-for-account";
        default: || Some(true);
    }
}
