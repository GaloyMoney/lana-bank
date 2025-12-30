#[cfg(feature = "sumsub-testing")]
use sumsub::testing_utils;

/// Creates a real Sumsub applicant via API (requires environment setup)
#[cfg(feature = "sumsub-testing")]
#[tokio::test]
async fn test_create_real_applicant() {
    use sumsub::{SumsubClient, testing::*};
    use uuid::Uuid;

    // Check if environment variables are set, skip test if not
    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!("Skipping test: SUMSUB_KEY and SUMSUB_SECRET must be set");
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
#[cfg(feature = "sumsub-testing")]
#[tokio::test]
async fn test_create_permalink() {
    use sumsub::SumsubClient;
    use uuid::Uuid;

    // Check if environment variables are set, skip test if not
    let config = match testing_utils::load_config_from_env() {
        Some(config) => config,
        None => {
            eprintln!("Skipping test: SUMSUB_KEY and SUMSUB_SECRET must be set");
            return;
        }
    };

    let client = SumsubClient::new(&config);
    let external_user_id = Uuid::new_v4().to_string();

    let permalink = client
        .create_permalink(&external_user_id, testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create permalink");

   assert!(
        permalink.url.starts_with("https://api.sumsub.com/"),
        "permalink must start with the official Sumsub API domain: {}",
        permalink.url
    );
    
     assert!(
        parsed.path().ends_with("/websdkLink"),
        "permalink path must end with /websdkLink, got: {}",
        parsed.path()
    );
}
