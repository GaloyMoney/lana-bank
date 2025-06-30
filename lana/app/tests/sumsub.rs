mod helpers;

use lana_app::{applicant::*, customer::CustomerId};

use std::env;

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
    }

    // Create SumsubClient directly - no need for full app initialization
    let sumsub_client = SumsubClient::new(&sumsub_config.unwrap());

    // Generate a random test customer ID
    let customer_id = CustomerId::new();

    // Test creating a permalink directly
    match sumsub_client
        .create_permalink(customer_id, "basic-kyc-level")
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
