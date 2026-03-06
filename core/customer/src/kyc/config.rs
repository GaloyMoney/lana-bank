use domain_config::define_exposed_config;

define_exposed_config! {
    /// SumSub API key for KYC verification
    pub struct SumsubApiKey(String);
    spec {
        key: "sumsub-api-key";
        encrypted: true;
    }
}

define_exposed_config! {
    /// SumSub API secret for KYC verification
    pub struct SumsubApiSecret(String);
    spec {
        key: "sumsub-api-secret";
        encrypted: true;
    }
}

define_exposed_config! {
    /// SumSub flow name for KYC (individual) verification
    pub struct SumsubKycFlowName(String);
    spec {
        key: "sumsub-kyc-flow-name";
        default: || Some("basic-kyc-level".to_string());
    }
}

define_exposed_config! {
    /// SumSub flow name for KYB (business) verification
    pub struct SumsubKybFlowName(String);
    spec {
        key: "sumsub-kyb-flow-name";
        default: || Some("kyb-basic".to_string());
    }
}
