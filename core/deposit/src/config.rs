use domain_config::define_exposed_config;

define_exposed_config! {
    pub struct RequireVerifiedCustomerForAccount(bool);
    spec {
        key: "require-verified-customer-for-account";
        default: || Some(true);
    }
}
