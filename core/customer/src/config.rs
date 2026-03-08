use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether admin manual conversion of prospects to customers is allowed.
    /// When enabled, admins can convert prospects without SumSub KYC approval.
    /// When disabled (default), prospects must go through SumSub KYC verification.
    pub struct AllowManualConversion(bool);
    spec {
        key: "allow-manual-conversion";
        default: || Some(false);
    }
}
