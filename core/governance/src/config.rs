use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether policies with AutoApprove approval rules can be created or updated.
    /// When enabled, policies with automatic approval can be created or updated.
    /// When disabled, all policies must use committee approval.
    pub struct AllowAutoApproval(bool);
    spec {
        key: "allow-system-auto-approval";
        default: || Some(true);
    }
}
