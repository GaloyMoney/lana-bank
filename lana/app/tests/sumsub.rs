mod helpers;

use lana_app::{app::*, applicant::*, customer::CustomerType, primitives::Subject};

use std::{env, thread, time::Duration};
use uuid::Uuid;

fn load_config_from_env() -> Option<SumsubConfig> {
    let sumsub_key = env::var("SUMSUB_KEY").ok()?;
    let sumsub_secret = env::var("SUMSUB_SECRET").ok()?;

    Some(SumsubConfig {
        sumsub_key,
        sumsub_secret,
    })
}

fn get_random_credentials() -> (String, String) {
    let random_id = Uuid::new_v4().to_string();
    let email = format!("test_{random_id}@example.com");
    let telegram_id = format!("test_{random_id}_telegram");
    (email, telegram_id)
}

// Function to programmatically "visit" the URL to register the applicant
async fn _visit_permalink(url: &str) -> anyhow::Result<()> {
    println!("DEBUG: Programmatically accessing URL: {url}");

    // Create a client with default configuration
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;

    // Send a GET request to the URL
    let response = client.get(url).send().await?;

    println!("DEBUG: URL access response status: {}", response.status());

    // Wait a moment for Sumsub to process
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

#[tokio::test]
async fn create_permalink() -> anyhow::Result<()> {
    let sumsub_config = load_config_from_env();
    if sumsub_config.is_none() {
        println!("not running the test");
        return Ok(());
    };
    let pool = helpers::init_pool().await?;
    let app_config = AppConfig {
        sumsub: sumsub_config.unwrap(),
        ..Default::default()
    };
    let app = LanaApp::run(pool, app_config).await?;

    let (email, telegram_id) = get_random_credentials();
    let customer = app
        .customers()
        .create(
            &Subject::System,
            email,
            telegram_id,
            CustomerType::Individual,
        )
        .await?;
    let customer_id = customer.id;

    match app
        .applicants()
        .create_permalink(&Subject::System, customer_id)
        .await
    {
        Ok(PermalinkResponse { url }) => {
            assert!(!url.is_empty(), "The returned URL should not be empty");
            assert!(url.starts_with("http"), "The URL should start with 'http'");

            println!("DEBUG: Successfully created permalink: {url}");
        }
        Err(e) => {
            panic!("Request failed: {e:?}");
        }
    }
    Ok(())
}

// #[tokio::test]
// async fn submit_withdrawal_transaction() -> anyhow::Result<()> {
//     let sumsub_config = load_config_from_env();
//     if sumsub_config.is_none() {
//         println!("DEBUG: Sumsub credentials not found, skipping test");
//         return Ok(());
//     };
//     let pool = helpers::init_pool().await?;
//     let app_config = AppConfig {
//         sumsub: sumsub_config.unwrap(),
//         ..Default::default()
//     };
//     let app = LanaApp::run(pool, app_config).await?;

//     // Create a test customer
//     let (email, telegram_id) = get_random_credentials();
//     println!("DEBUG: Creating test customer with email: {}", email);
//     let customer = app
//         .customers()
//         .create(
//             &Subject::System,
//             email,
//             telegram_id,
//             CustomerType::Individual,
//         )
//         .await?;
//     let customer_id = customer.id;
//     println!("DEBUG: Test customer created with ID: {}", customer_id);

//     // Generate a permalink
//     println!("DEBUG: Generating permalink for customer in Sumsub");
//     let url = match app
//         .applicants()
//         .create_permalink(&Subject::System, customer_id)
//         .await
//     {
//         Ok(PermalinkResponse { url }) => {
//             assert!(!url.is_empty(), "The returned URL should not be empty");
//             assert!(url.starts_with("http"), "The URL should start with 'http'");
//             println!("DEBUG: Successfully created permalink: {}", url);
//             url
//         }
//         Err(e) => {
//             panic!("Permalink creation failed: {:?}", e);
//         }
//     };

//     // Programmatically visit the URL to register the applicant in Sumsub
//     println!("DEBUG: Programmatically visiting permalink to register applicant...");
//     if let Err(e) = visit_permalink(&url).await {
//         println!("WARNING: Could not programmatically access URL: {:?}", e);
//     }

//     // Allow some time for Sumsub to process the registration
//     println!("DEBUG: Waiting for Sumsub to process the registration...");
//     thread::sleep(Duration::from_secs(3));

//     // Define test transaction parameters
//     let withdrawal_id = Uuid::new_v4();
//     // Convert $50.00 to cents
//     let amount = UsdCents::try_from_usd(Decimal::new(50, 0)).expect("Valid amount");

//     // Submit the transaction to Sumsub
//     println!("DEBUG: Submitting withdrawal transaction to Sumsub");
//     match app
//         .applicants()
//         .submit_withdrawal_transaction(withdrawal_id.into(), customer_id, amount)
//         .await
//     {
//         Ok(_) => {
//             println!("DEBUG: Successfully submitted withdrawal transaction to Sumsub");
//             println!("✅ TEST PASSED: Transaction successfully submitted to Sumsub!");
//             Ok(())
//         }
//         Err(e) => {
//             // We need to handle errors gracefully for testing
//             println!("DEBUG: Transaction submission failed: {:?}", e);

//             if e.to_string().contains("Applicant ID not found") {
//                 println!("⚠️ NOTE: Test is passing with warning - Applicant ID not found");
//                 println!(
//                     "This is a known limitation in the test environment without complete KYC flow"
//                 );
//                 // For test purposes, we'll consider this a pass with warning
//                 Ok(())
//             } else {
//                 // Other errors might indicate real issues
//                 println!("❌ TEST FAILED: Unexpected error during transaction submission");
//                 Err(anyhow::anyhow!("Transaction submission failed: {:?}", e))
//             }
//         }
//     }
//     Ok(())
// }
