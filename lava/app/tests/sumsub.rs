mod helpers;

use lava_app::{app::*, applicant::*, primitives::CustomerId};

use std::env;

fn load_config_from_env() -> Option<SumsubConfig> {
    let sumsub_key = env::var("SUMSUB_KEY").ok()?;
    let sumsub_secret = env::var("SUMSUB_SECRET").ok()?;

    Some(SumsubConfig {
        sumsub_key,
        sumsub_secret,
    })
}

#[tokio::test]
async fn get_access_token() -> anyhow::Result<()> {
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
    let app = LavaApp::run(pool, app_config).await?;

    let customer_id = CustomerId::new();
    match app.applicants().create_access_token(customer_id).await {
        Ok(AccessTokenResponse {
            token,
            customer_id: returned_customer_id,
        }) => {
            assert!(!token.is_empty(), "The returned token should not be empty");
            assert_eq!(
                customer_id.to_string(),
                returned_customer_id,
                "The returned customer_id should match the input customer_id"
            );
        }
        Err(e) => {
            panic!("Request failed: {:?}", e);
        }
    }
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
    let app = LavaApp::run(pool, app_config).await?;

    let customer_id = CustomerId::new();
    match app.applicants().create_permalink(customer_id).await {
        Ok(PermalinkResponse { url }) => {
            assert!(!url.is_empty(), "The returned URL should not be empty");
            assert!(url.starts_with("http"), "The URL should start with 'http'");
        }
        Err(e) => {
            panic!("Request failed: {:?}", e);
        }
    }
    Ok(())
}
