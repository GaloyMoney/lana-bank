use keycloak_admin::{KeycloakAdmin, KeycloakConnectionConfig};
use uuid::Uuid;

#[tokio::test]
async fn test_create_user() {
    let config = KeycloakConnectionConfig::default();
    let admin = KeycloakAdmin::new(config, "internal".to_string());
    let test_email = format!("test-user-{}@example.com", Uuid::new_v4());
    let user_id = admin
        .create_user(test_email.clone())
        .await
        .expect("Failed to create user");

    assert!(user_id != Uuid::nil(), "User ID should be valid");
}

#[tokio::test]
async fn test_update_user_email() {
    let config = KeycloakConnectionConfig::default();
    let admin = KeycloakAdmin::new(config, "internal".to_string());
    let initial_email = format!("test-user-initial-{}@example.com", Uuid::new_v4());
    let updated_email = format!("test-user-updated-{}@example.com", Uuid::new_v4());
    let user_id = admin
        .create_user(initial_email)
        .await
        .expect("Failed to create user");
    admin
        .update_user_email(user_id, updated_email)
        .await
        .expect("Failed to update user email");
}

#[tokio::test]
async fn test_get_user() {
    let config = KeycloakConnectionConfig::default();
    let admin = KeycloakAdmin::new(config, "internal".to_string());
    let test_email = format!("test-get-user-{}@example.com", Uuid::new_v4());
    let user_id = admin
        .create_user(test_email.clone())
        .await
        .expect("Failed to create user");
    let user = admin.get_user(user_id).await.expect("Failed to get user");
    assert_eq!(user.email, Some(test_email));
    assert_eq!(user.enabled, Some(true));
    assert_eq!(user.email_verified, Some(true));
}
