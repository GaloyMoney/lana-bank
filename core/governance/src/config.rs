use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether policies with SystemAutoApprove approval rules can be created or updated.
    /// When enabled, creating new policies with SystemAutoApprove and updating existing
    /// policies to SystemAutoApprove are both prohibited.
    pub struct RequireCommitteeApproval(bool);
    spec {
        key: "require-committee-approval";
        default: || Some(false);
    }
}
