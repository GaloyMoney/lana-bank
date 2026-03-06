use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether manual collateral updates are disabled.
    /// When set to true, admin users cannot manually update collateral amounts
    /// via the admin API. Custodian sync and liquidation updates are unaffected.
    pub struct DisableManualCollateral(bool);
    spec {
        key: "disable-manual-collateral";
        default: || Some(false);
    }
}
