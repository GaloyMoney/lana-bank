#[cfg(feature = "sumsub-testing")]
use core_applicant::sumsub_testing_utils;

/// Creates a real Sumsub applicant via API (requires environment setup)
#[cfg(feature = "sumsub-testing")]
#[tokio::test]
async fn test_create_real_applicant() {
    use core_applicant::SumsubClient;
    use core_customer::CustomerId;

    // Load configuration from environment
    let config = sumsub_testing_utils::load_config_from_env()
        .expect("SUMSUB_KEY and SUMSUB_SECRET must be set");

    // Create client
    let client = SumsubClient::new(&config);

    // Create a test customer ID
    let customer_id = CustomerId::new();

    let applicant_id = client
        .create_applicant(customer_id, sumsub_testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create applicant");

    client
        .update_applicant_info(
            &applicant_id,
            sumsub_testing_utils::TEST_FIRST_NAME,
            sumsub_testing_utils::TEST_LAST_NAME,
            sumsub_testing_utils::TEST_DATE_OF_BIRTH,
            sumsub_testing_utils::TEST_COUNTRY_CODE,
        )
        .await
        .expect("Failed to update applicant info");

    let applicant_details = client
        .get_applicant_details(customer_id)
        .await
        .expect("Failed to get applicant details");

    assert_eq!(applicant_details.customer_id, customer_id);
    assert_eq!(
        applicant_details.fixed_info.first_name(),
        Some(sumsub_testing_utils::TEST_FIRST_NAME)
    );
    assert_eq!(
        applicant_details.fixed_info.last_name(),
        Some(sumsub_testing_utils::TEST_LAST_NAME)
    );
    assert_eq!(
        applicant_details.fixed_info.full_name(),
        Some(format!(
            "{} {}",
            sumsub_testing_utils::TEST_FIRST_NAME,
            sumsub_testing_utils::TEST_LAST_NAME
        ))
    );
}

/// Creates a permalink for KYC flow (requires environment setup)
#[cfg(feature = "sumsub-testing")]
#[tokio::test]
async fn test_create_permalink() {
    use core_applicant::SumsubClient;
    use core_customer::CustomerId;

    let config = sumsub_testing_utils::load_config_from_env()
        .expect("SUMSUB_KEY and SUMSUB_SECRET must be set");
    let client = SumsubClient::new(&config);
    let customer_id = CustomerId::new();

    let permalink = client
        .create_permalink(customer_id, sumsub_testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create permalink");

    assert!(permalink.url.contains("sumsub.com"));
    assert!(permalink.url.contains("websdk"));
}
