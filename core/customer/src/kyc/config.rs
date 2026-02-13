use domain_config::define_exposed_config;

define_exposed_config! {
    /// SumSub API key for KYC verification
    pub struct SumsubApiKey(String);
    spec {
        key: "sumsub-api-key";
        encrypted: true;
        default: || Some(String::new());
    }
}

define_exposed_config! {
    /// SumSub API secret for KYC verification
    pub struct SumsubApiSecret(String);
    spec {
        key: "sumsub-api-secret";
        encrypted: true;
        default: || Some(String::new());
    }
}
