use core_applicant::SumsubConfig;

// Test configuration constants
#[allow(dead_code)]
pub const TEST_LEVEL_NAME: &str = "basic-kyc-level";
#[allow(dead_code)]
pub const TEST_FIRST_NAME: &str = "John";
#[allow(dead_code)]
pub const TEST_LAST_NAME: &str = "Mock-Doe";
#[allow(dead_code)]
pub const TEST_DATE_OF_BIRTH: &str = "1990-01-01";
#[allow(dead_code)]
pub const TEST_COUNTRY_CODE: &str = "DEU";
#[allow(dead_code)]
pub const TEST_QUESTIONNAIRE_ID: &str = "v1_onboarding";
#[allow(dead_code)]
pub const TEST_CURRENCY: &str = "USD";
#[allow(dead_code)]
pub const TEST_TX_TYPE: &str = "deposit";
#[allow(dead_code)]
pub const TEST_TX_DIRECTION: &str = "incoming";
#[allow(dead_code)]
pub const TEST_TX_AMOUNT: f64 = 1000.0;

// Test artifact filenames
#[allow(dead_code)]
pub const PASSPORT_FILENAME: &str = "german_passport.jpg";
#[allow(dead_code)]
pub const POA_FILENAME: &str = "poa_krause_green.jpg";

// Test document URLs
#[allow(dead_code)]
pub const GERMAN_PASSPORT_URL: &str = "https://sumsub.com/files/29346237-germany-passport.jpg";
#[allow(dead_code)]
pub const POA_DOCUMENT_URL: &str = "https://sumsub.com/files/62349849-poa-krause-green.jpg";

/// Generic function to load test documents, downloading if not present locally
pub async fn load_test_document(
    filename: &str,
    download_url: &str,
    description: &str,
) -> Result<Vec<u8>, std::io::Error> {
    let artefacts_dir = "artefacts";
    let image_path = format!("{artefacts_dir}/{filename}");

    // Check if file already exists locally
    if std::path::Path::new(&image_path).exists() {
        return std::fs::read(&image_path);
    }

    // File doesn't exist, download it
    println!("Downloading {description} for testing...");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(artefacts_dir)?;

    // Download the image
    let image_data = reqwest::get(download_url)
        .await
        .map_err(|e| std::io::Error::other(format!("Download failed: {e}")))?
        .bytes()
        .await
        .map_err(|e| std::io::Error::other(format!("Failed to read bytes: {e}")))?;

    // Save to local file
    std::fs::write(&image_path, &image_data)?;
    println!("{description} downloaded and saved to {image_path}");

    Ok(image_data.to_vec())
}

pub fn load_config_from_env() -> Option<SumsubConfig> {
    let sumsub_key = std::env::var("SUMSUB_KEY").ok()?;
    let sumsub_secret = std::env::var("SUMSUB_SECRET").ok()?;

    Some(SumsubConfig {
        sumsub_key,
        sumsub_secret,
    })
}

/// Creates a real Sumsub applicant via API (requires environment setup)
#[cfg(feature = "sumsub-testing")]
#[tokio::test]
async fn test_create_real_applicant() {
    use core_applicant::{sumsub_testing_utils, SumsubClient};
    use core_customer::CustomerId;

    // Load configuration from environment
    let config = load_config_from_env().expect("SUMSUB_KEY and SUMSUB_SECRET must be set");

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
    use core_applicant::{sumsub_testing_utils, SumsubClient};
    use core_customer::CustomerId;

    let config = load_config_from_env().expect("SUMSUB_KEY and SUMSUB_SECRET must be set");
    let client = SumsubClient::new(&config);
    let customer_id = CustomerId::new();

    let permalink = client
        .create_permalink(customer_id, sumsub_testing_utils::TEST_LEVEL_NAME)
        .await
        .expect("Failed to create permalink");

    assert!(permalink.url.contains("sumsub.com"));
    assert!(permalink.url.contains("websdk"));
}
