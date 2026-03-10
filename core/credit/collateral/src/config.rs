use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether manual collateral updates are enabled.
    /// Manual collateral is intended for testing purposes only.
    /// When set to false, admin users cannot manually update collateral amounts
    /// via the admin API. Custodian sync and liquidation updates are unaffected.
    pub struct ManualCollateral(bool);
    spec {
        key: "manual-collateral";
        default: || Some(false);
    }
}
