use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether manual custodian is enabled.
    /// Manual custodian is intended for testing purposes only.
    /// When set to false, admin users cannot select manual custodian
    /// when creating credit facility proposals.
    pub struct EnableManualCustodian(bool);
    spec {
        key: "enable-manual-custodian";
        default: || Some(false);
    }
}
