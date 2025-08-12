use keycloak_client::{KeycloakClient, KeycloakConnectionConfig};
use uuid::Uuid;

#[tokio::test]
async fn test_create_user() {
    let config = KeycloakConnectionConfig::default();
    let admin = KeycloakClient::new(config);
    let test_email = format!("test-user-{}@example.com", Uuid::new_v4());
    let user_id = admin
        .create_user(test_email.clone(), Uuid::new_v4())
        .await
        .expect("Failed to create user");

    assert!(user_id != Uuid::nil(), "User ID should be valid");
}

#[tokio::test]
async fn test_update_user_email() {
    let config = KeycloakConnectionConfig::default();
    let lana_user_id = Uuid::new_v4();
    let admin = KeycloakClient::new(config);
    let initial_email = format!("test-user-initial-{}@example.com", lana_user_id);
    let updated_email = format!("test-user-updated-{}@example.com", lana_user_id);
    admin
        .create_user(initial_email, lana_user_id)
        .await
        .expect("Failed to create user");
    admin
        .update_user_email(lana_user_id, updated_email)
        .await
        .expect("Failed to update user email");
}

#[tokio::test]
async fn test_get_keycloak_id_by_lana_id() {
    let config = KeycloakConnectionConfig::default();
    let admin = KeycloakClient::new(config);
    let lana_user_id = Uuid::new_v4();
    let test_email = format!("test-get-user-{}@example.com", lana_user_id);
    let user_id = admin
        .create_user(test_email.clone(), lana_user_id)
        .await
        .expect("Failed to create user");
    let keycloak_user_id = admin
        .get_keycloak_id_by_lana_id(lana_user_id)
        .await
        .expect("Failed to get user");
    assert_eq!(keycloak_user_id, user_id);
}
