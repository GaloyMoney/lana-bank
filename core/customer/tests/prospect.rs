mod helpers;

use authz::dummy::DummySubject;
use uuid::Uuid;

use core_customer::{
    CoreCustomerEvent, CustomerId, CustomerType, KycStatus, KycVerification, ProspectStatus,
};
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
    assert_eq!(recorded.status, ProspectStatus::Open);
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
/// is published with kyc_status set to Approved and status set to Converted.
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

    // Start KYC first (required before approval)
    customers
        .handle_kyc_started(prospect.id, applicant_id.clone())
        .await?;

    let (_created_customer, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_approved(prospect.id, applicant_id.clone()),
        |_result, e| match e {
            CoreCustomerEvent::ProspectKycUpdated { entity }
                if entity.id == prospect.id && entity.kyc_status == KycStatus::Approved =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, prospect.id);
    assert_eq!(recorded.kyc_status, KycStatus::Approved);
    assert_eq!(recorded.status, ProspectStatus::Converted);

    Ok(())
}

/// ProspectClosed event is published when a prospect is closed
/// via `Customers::close_prospect()`.
///
/// This is an operator-driven action to manually close a prospect
/// that is no longer needed.
///
/// The event contains a snapshot with status set to Closed.
#[tokio::test]
async fn prospect_closed_event_on_close_prospect() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;

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

    let (_closed_prospect, recorded) = event::expect_event(
        &outbox,
        || customers.close_prospect(&DummySubject, prospect.id),
        |result, e| match e {
            CoreCustomerEvent::ProspectClosed { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, prospect.id);
    assert_eq!(recorded.status, ProspectStatus::Closed);

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

    // Start KYC first (required before decline)
    customers
        .handle_kyc_started(prospect.id, applicant_id.clone())
        .await?;

    let (updated_prospect, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_declined(prospect.id, applicant_id.clone()),
        |result, e| match e {
            CoreCustomerEvent::ProspectKycUpdated { entity }
                if entity.id == result.id && entity.kyc_status == KycStatus::Declined =>
            {
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

/// When a prospect has been approved and converted to a customer, a subsequent
/// SumSub decline callback should update the Customer's kyc_verification to Rejected
/// instead of modifying the Prospect's kyc_status.
///
/// This ensures that post-conversion KYC rejections are properly routed to the
/// Customer entity and emit a CustomerKycRejected event.
#[tokio::test]
async fn decline_after_approval_updates_customer_not_prospect() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;

    // Create a prospect and approve KYC to convert to customer
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
    let customer = customers
        .handle_kyc_approved(prospect.id, applicant_id.clone())
        .await?;

    assert_eq!(customer.kyc_verification, KycVerification::Verified);

    // Now decline KYC (simulating a SumSub RED callback after approval)
    let (_prospect_returned, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_declined(prospect.id, applicant_id.clone()),
        |_result, e| match e {
            CoreCustomerEvent::CustomerKycRejected { entity }
                if entity.id == CustomerId::from(prospect.id) =>
            {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    // Customer should be rejected
    assert_eq!(recorded.kyc_verification, KycVerification::Rejected);

    // Prospect kyc_status should remain Approved (not changed to Declined)
    let prospect_after = customers
        .find_prospect_by_id(&DummySubject, prospect.id)
        .await?
        .expect("prospect should still exist");
    assert_eq!(prospect_after.kyc_status, KycStatus::Approved);
    assert_eq!(prospect_after.status, ProspectStatus::Converted);

    Ok(())
}
