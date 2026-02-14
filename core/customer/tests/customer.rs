mod helpers;

use authz::dummy::DummySubject;
use uuid::Uuid;

use core_customer::{CoreCustomerEvent, CustomerType, KycVerification};
use helpers::event;

/// CustomerCreated event is published when a prospect's KYC is approved
/// via `Customers::handle_kyc_approved()`.
///
/// This typically happens when an external KYC provider (e.g., SumSub) notifies
/// the system that identity verification has passed. The prospect is converted
/// into a customer.
///
/// The event contains a snapshot of the newly created customer with kyc_verification set to Verified.
#[tokio::test]
async fn customer_created_event_on_kyc_approved() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;

    // First create a prospect
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_handle = format!("telegram-{}", Uuid::new_v4());
    let prospect = customers
        .create_prospect(
            &DummySubject,
            email,
            telegram_handle,
            CustomerType::Individual,
        )
        .await?;

    let applicant_id = format!("applicant-{}", Uuid::new_v4());

    // Start KYC first (required before approval)
    customers
        .handle_kyc_started(
            prospect.id,
            applicant_id.clone(),
            "test-callback".to_string(),
        )
        .await?;

    let (created_customer, recorded) = event::expect_event(
        &outbox,
        || {
            customers.handle_kyc_approved(
                prospect.id,
                applicant_id.clone(),
                "test-callback".to_string(),
            )
        },
        |result, e| match e {
            CoreCustomerEvent::CustomerCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, created_customer.id);
    assert_eq!(recorded.kyc_verification, KycVerification::Verified);

    Ok(())
}

/// CustomerEmailUpdated event is published when a customer's email is changed
/// via `Customers::update_email()`.
///
/// This event is consumed by downstream systems (e.g., Keycloak) to keep
/// authentication credentials in sync with customer data.
///
/// The event contains a snapshot with the new email address.
#[tokio::test]
async fn customer_email_updated_event_on_email_change() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;

    // First create a customer
    let original_email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_handle = format!("telegram-{}", Uuid::new_v4());
    let customer = customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            original_email,
            telegram_handle,
            CustomerType::Individual,
        )
        .await?;

    let new_email = format!("updated-{}@example.com", Uuid::new_v4());

    let (updated_customer, recorded) = event::expect_event(
        &outbox,
        || customers.update_email(&DummySubject, customer.id, new_email.clone()),
        |result, e| match e {
            CoreCustomerEvent::CustomerEmailUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_customer.id);
    assert_eq!(recorded.email, new_email);

    Ok(())
}
