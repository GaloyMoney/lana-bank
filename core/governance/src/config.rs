use domain_config::define_exposed_config;

define_exposed_config! {
    /// Controls whether policies with SystemAutoApprove are allowed to start approval processes.
    /// When enabled, all policies must have a committee assigned before approval processes can start.
    pub struct RequireCommitteeApproval(bool);
    spec {
        key: "require-committee-approval";
        default: || Some(false);
    }
}
