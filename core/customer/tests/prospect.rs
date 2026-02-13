mod helpers;

use authz::dummy::DummySubject;
use uuid::Uuid;

use core_customer::{CoreCustomerEvent, CustomerType, KycStatus};
use helpers::event;

/// ProspectCreated event is published when a new prospect is created
/// via `Customers::create_prospect()`.
///
/// This is consumed by Keycloak sync to create a user account for the prospect
/// before they complete KYC verification.
///
/// The event contains a snapshot of the newly created prospect.
#[tokio::test]
async fn prospect_created_event_on_create_prospect() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;

    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_handle = format!("telegram-{}", Uuid::new_v4());

    let (created_prospect, recorded) = event::expect_event(
        &outbox,
        || {
            customers.create_prospect(
                &DummySubject,
                email.clone(),
                telegram_handle.clone(),
                CustomerType::Individual,
            )
        },
        |result, e| match e {
            CoreCustomerEvent::ProspectCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, created_prospect.id);
    assert_eq!(recorded.email, email);
    assert_eq!(recorded.kyc_status, KycStatus::NotStarted);

    Ok(())
}

/// ProspectKycUpdated event is published when a prospect starts KYC
/// via `Customers::handle_kyc_started()`.
///
/// This typically happens when an external KYC provider (e.g., SumSub) notifies
/// the system that identity verification has begun.
///
/// The event contains a snapshot with kyc_status set to Pending.
#[tokio::test]
async fn prospect_kyc_updated_event_on_kyc_started() -> anyhow::Result<()> {
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

    let (updated_prospect, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_started(prospect.id, applicant_id.clone()),
        |result, e| match e {
            CoreCustomerEvent::ProspectKycUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_prospect.id);
    assert_eq!(recorded.kyc_status, KycStatus::Pending);

    Ok(())
}

/// ProspectKycUpdated event is published when a prospect's KYC is approved
/// via `Customers::handle_kyc_approved()`.
///
/// Note: This also triggers a CustomerCreated event since the prospect is
/// converted to a customer. This test verifies the ProspectKycUpdated event
/// is published with kyc_status set to Closed.
#[tokio::test]
async fn prospect_kyc_updated_event_on_kyc_approved() -> anyhow::Result<()> {
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

    let (_created_customer, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_approved(prospect.id, applicant_id.clone()),
        |_result, e| match e {
            CoreCustomerEvent::ProspectKycUpdated { entity } if entity.id == prospect.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, prospect.id);
    assert_eq!(recorded.kyc_status, KycStatus::Closed);

    Ok(())
}

/// ProspectKycUpdated event is published when a prospect's KYC is declined
/// via `Customers::handle_kyc_declined()`.
///
/// This typically happens when an external KYC provider notifies the system
/// that identity verification has failed.
///
/// The event contains a snapshot with kyc_status set to Declined.
#[tokio::test]
async fn prospect_kyc_updated_event_on_kyc_declined() -> anyhow::Result<()> {
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

    let (updated_prospect, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_declined(prospect.id, applicant_id.clone()),
        |result, e| match e {
            CoreCustomerEvent::ProspectKycUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_prospect.id);
    assert_eq!(recorded.kyc_status, KycStatus::Declined);

    Ok(())
}
