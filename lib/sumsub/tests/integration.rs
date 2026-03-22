use sumsub::testing_utils;

/// Creates a real Sumsub applicant via API (requires environment setup)
#[tokio::test]
async fn test_create_real_applicant() {
    use sumsub::{SumsubClient, testing::*};
    use uuid::Uuid;

    // Check if environment variables are set, skip test if not
    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!(
                "Skipping test: LANA_DOMAIN_CONFIG_SUMSUB_API_KEY and LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET must be set"
            );
            return;
        }
    };

    // Create client
    let client = SumsubClient::new(&config);

    // Create a test external user ID (using UUID as a string)
    let external_user_id = Uuid::new_v4().to_string();

    let applicant_id = client
        .create_applicant(&external_user_id, testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create applicant");

    client
        .update_applicant_info(
            &applicant_id,
            TEST_FIRST_NAME,
            TEST_LAST_NAME,
            TEST_DATE_OF_BIRTH,
            TEST_COUNTRY_CODE,
        )
        .await
        .expect("Failed to update applicant info");

    let applicant_details = client
        .get_applicant_details(external_user_id.clone())
        .await
        .expect("Failed to get applicant details");

    assert_eq!(applicant_details.external_user_id, external_user_id);
    assert_eq!(
        applicant_details.fixed_info.first_name(),
        Some(TEST_FIRST_NAME)
    );
    assert_eq!(
        applicant_details.fixed_info.last_name(),
        Some(TEST_LAST_NAME)
    );
    assert_eq!(
        applicant_details.fixed_info.full_name(),
        Some(format!("{TEST_FIRST_NAME} {TEST_LAST_NAME}"))
    );
}

/// Creates a permalink for KYC flow (requires environment setup)
#[tokio::test]
async fn test_create_permalink() {
    use sumsub::SumsubClient;
    use uuid::Uuid;

    // Check if environment variables are set, skip test if not
    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!(
                "Skipping test: LANA_DOMAIN_CONFIG_SUMSUB_API_KEY and LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET must be set"
            );
            return;
        }
    };

    let client = SumsubClient::new(&config);
    let external_user_id = Uuid::new_v4().to_string();

    let permalink = client
        .create_permalink(&external_user_id, testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create permalink");

    assert!(permalink.url.contains("sumsub.com"));
    assert!(permalink.url.contains("websdk"));
}

/// Creates a test applicant, approves it, then rejects it via the production reject endpoint
#[tokio::test]
async fn test_reject_applicant() {
    use sumsub::{SumsubClient, testing::*};
    use uuid::Uuid;

    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!(
                "Skipping test: LANA_DOMAIN_CONFIG_SUMSUB_API_KEY and LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET must be set"
            );
            return;
        }
    };

    let client = SumsubClient::new(&config);
    let external_user_id = Uuid::new_v4().to_string();

    // Create and approve applicant first
    let applicant_id = client
        .create_applicant(&external_user_id, testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create applicant");

    client
        .update_applicant_info(
            &applicant_id,
            TEST_FIRST_NAME,
            TEST_LAST_NAME,
            TEST_DATE_OF_BIRTH,
            TEST_COUNTRY_CODE,
        )
        .await
        .expect("Failed to update applicant info");

    client
        .simulate_review_response(&applicant_id, REVIEW_ANSWER_GREEN)
        .await
        .expect("Failed to approve applicant");

    // Now reject via the production endpoint
    client
        .reject_applicant(external_user_id, "Customer account frozen by compliance")
        .await
        .expect("Failed to reject applicant");
}

/// Rejects a test applicant, then restores it via the production approve endpoint.
#[tokio::test]
async fn test_approve_applicant() {
    use sumsub::{SumsubClient, testing::*};
    use uuid::Uuid;

    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!(
                "Skipping test: LANA_DOMAIN_CONFIG_SUMSUB_API_KEY and LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET must be set"
            );
            return;
        }
    };

    let client = SumsubClient::new(&config);
    let external_user_id = Uuid::new_v4().to_string();

    let applicant_id = client
        .create_applicant(&external_user_id, testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create applicant");

    client
        .update_applicant_info(
            &applicant_id,
            TEST_FIRST_NAME,
            TEST_LAST_NAME,
            TEST_DATE_OF_BIRTH,
            TEST_COUNTRY_CODE,
        )
        .await
        .expect("Failed to update applicant info");

    client
        .simulate_review_response(&applicant_id, REVIEW_ANSWER_GREEN)
        .await
        .expect("Failed to approve applicant");

    client
        .reject_applicant(
            external_user_id.clone(),
            "Customer account frozen by compliance",
        )
        .await
        .expect("Failed to reject applicant");

    client
        .approve_applicant(external_user_id)
        .await
        .expect("Failed to approve applicant");
}
