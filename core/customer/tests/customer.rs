mod helpers;

use authz::dummy::DummySubject;
use uuid::Uuid;

use core_customer::{CoreCustomerEvent, CustomerStatus, CustomerType, PersonalInfo};
use helpers::event;

/// CustomerCreated event is published when a prospect's KYC is approved
/// via `Customers::handle_kyc_approved()`.
///
/// This typically happens when an external KYC provider (e.g., SumSub) notifies
/// the system that identity verification has passed. The prospect is converted
/// into a customer.
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
        .handle_kyc_started(prospect.id, applicant_id.clone())
        .await?;

    let (created_customer, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_approved(prospect.id, applicant_id.clone(), PersonalInfo::dummy()),
        |result, e| match e {
            CoreCustomerEvent::CustomerCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, created_customer.id);

    Ok(())
}

/// PartyEmailUpdated event is published when a customer's email is changed
/// via `Customers::update_email()`.
///
/// This event is consumed by downstream systems (e.g., Keycloak) to keep
/// authentication credentials in sync with customer data.
///
/// The event contains a snapshot with the new email address.
#[tokio::test]
async fn party_email_updated_event_on_email_change() -> anyhow::Result<()> {
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

    let (_, recorded) = event::expect_event(
        &outbox,
        || customers.update_email(&DummySubject, customer.party_id, new_email.clone()),
        |result, e| match e {
            CoreCustomerEvent::PartyEmailUpdated { entity } if entity.id == result.party_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.email, new_email);

    Ok(())
}

async fn create_test_customer(
    customers: &core_customer::Customers<
        authz::dummy::DummyPerms<helpers::action::DummyAction, helpers::object::DummyObject>,
        helpers::event::DummyEvent,
    >,
) -> anyhow::Result<core_customer::Customer> {
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_handle = format!("telegram-{}", Uuid::new_v4());
    let customer = customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            email,
            telegram_handle,
            CustomerType::Individual,
        )
        .await?;
    Ok(customer)
}

/// Freezing an active customer emits a CustomerFrozen event
/// and sets the customer status to Frozen.
#[tokio::test]
async fn customer_frozen_event_on_freeze() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    let (frozen, recorded) = event::expect_event(
        &outbox,
        || customers.freeze_customer(&DummySubject, customer.id),
        |result, e| match e {
            CoreCustomerEvent::CustomerFrozen { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(frozen.status, CustomerStatus::Frozen);
    assert_eq!(recorded.id, frozen.id);

    Ok(())
}

/// Unfreezing a frozen customer emits a CustomerUnfrozen event
/// and restores the customer status to Active.
#[tokio::test]
async fn customer_unfrozen_event_on_unfreeze() -> anyhow::Result<()> {
    let (customers, outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    customers
        .freeze_customer(&DummySubject, customer.id)
        .await?;

    let (unfrozen, recorded) = event::expect_event(
        &outbox,
        || customers.unfreeze_customer(&DummySubject, customer.id),
        |result, e| match e {
            CoreCustomerEvent::CustomerUnfrozen { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(unfrozen.status, CustomerStatus::Active);
    assert_eq!(recorded.id, unfrozen.id);

    Ok(())
}

/// Freezing an already-frozen customer is idempotent — no error, no new event.
#[tokio::test]
async fn freeze_is_idempotent() -> anyhow::Result<()> {
    let (customers, _outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    customers
        .freeze_customer(&DummySubject, customer.id)
        .await?;
    let frozen_again = customers
        .freeze_customer(&DummySubject, customer.id)
        .await?;

    assert_eq!(frozen_again.status, CustomerStatus::Frozen);

    Ok(())
}

/// Unfreezing an active (non-frozen) customer is idempotent — no error.
#[tokio::test]
async fn unfreeze_active_customer_is_idempotent() -> anyhow::Result<()> {
    let (customers, _outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    let result = customers
        .unfreeze_customer(&DummySubject, customer.id)
        .await?;

    assert_eq!(result.status, CustomerStatus::Active);

    Ok(())
}

/// Cannot freeze a closed customer.
#[tokio::test]
async fn cannot_freeze_closed_customer() -> anyhow::Result<()> {
    let (customers, _outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    customers.close_customer(&DummySubject, customer.id).await?;

    let result = customers.freeze_customer(&DummySubject, customer.id).await;
    assert!(result.is_err());

    Ok(())
}

/// Cannot unfreeze a closed customer.
#[tokio::test]
async fn cannot_unfreeze_closed_customer() -> anyhow::Result<()> {
    let (customers, _outbox) = helpers::setup().await?;
    let customer = create_test_customer(&customers).await?;

    customers
        .freeze_customer(&DummySubject, customer.id)
        .await?;
    customers.close_customer(&DummySubject, customer.id).await?;

    let result = customers
        .unfreeze_customer(&DummySubject, customer.id)
        .await;
    assert!(result.is_err());

    Ok(())
}
