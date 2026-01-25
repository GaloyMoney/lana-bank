mod helpers;

use authz::dummy::DummySubject;
use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use uuid::Uuid;

use core_customer::{CoreCustomerEvent, CustomerType, Customers, KycVerification};
use helpers::{action, event, object};

/// Creates a test setup with all required dependencies for customer tests.
async fn setup() -> anyhow::Result<(
    Customers<
        authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
        event::DummyEvent,
    >,
    obix::Outbox<event::DummyEvent>,
)> {
    let pool = helpers::init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let public_ids = public_id::PublicIds::new(&pool);

    let customers = Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage,
        public_ids,
        clock.clone(),
    );

    Ok((customers, outbox))
}

/// CustomerCreated event is published when a new customer is created via `Customers::create()`.
///
/// The event contains a snapshot of the customer entity at creation time, including:
/// - id: The unique customer identifier
/// - email: The customer's email address
/// - customer_type: The type of customer (Individual, Bank, etc.)
/// - kyc_verification: Initial KYC status (PendingVerification)
#[tokio::test]
async fn customer_created_event_on_create() -> anyhow::Result<()> {
    let (customers, outbox) = setup().await?;

    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_id = format!("telegram-{}", Uuid::new_v4());

    let (customer, recorded) = event::expect_event(
        &outbox,
        || {
            customers.create(
                &DummySubject,
                email.clone(),
                telegram_id.clone(),
                CustomerType::Individual,
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

    assert_eq!(recorded.id, customer.id);
    assert_eq!(recorded.email, email);
    assert_eq!(recorded.customer_type, CustomerType::Individual);
    assert_eq!(
        recorded.kyc_verification,
        KycVerification::PendingVerification
    );

    Ok(())
}

/// CustomerKycUpdated event is published when a customer's KYC is approved
/// via `Customers::handle_kyc_approved()`.
///
/// This typically happens when an external KYC provider (e.g., SumSub) notifies
/// the system that identity verification has passed.
///
/// The event contains a snapshot with kyc_verification set to Verified.
#[tokio::test]
async fn customer_kyc_updated_event_on_kyc_approved() -> anyhow::Result<()> {
    let (customers, outbox) = setup().await?;

    // First create a customer
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_id = format!("telegram-{}", Uuid::new_v4());
    let customer = customers
        .create(&DummySubject, email, telegram_id, CustomerType::Individual)
        .await?;

    let applicant_id = format!("applicant-{}", Uuid::new_v4());

    let (updated_customer, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_approved(customer.id, applicant_id.clone()),
        |result, e| match e {
            CoreCustomerEvent::CustomerKycUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_customer.id);
    assert_eq!(recorded.kyc_verification, KycVerification::Verified);

    Ok(())
}

/// CustomerKycUpdated event is published when a customer's KYC is declined
/// via `Customers::handle_kyc_declined()`.
///
/// This typically happens when an external KYC provider notifies the system
/// that identity verification has failed.
///
/// The event contains a snapshot with kyc_verification set to Rejected.
#[tokio::test]
async fn customer_kyc_updated_event_on_kyc_declined() -> anyhow::Result<()> {
    let (customers, outbox) = setup().await?;

    // First create a customer
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_id = format!("telegram-{}", Uuid::new_v4());
    let customer = customers
        .create(&DummySubject, email, telegram_id, CustomerType::Individual)
        .await?;

    let applicant_id = format!("applicant-{}", Uuid::new_v4());

    let (updated_customer, recorded) = event::expect_event(
        &outbox,
        || customers.handle_kyc_declined(customer.id, applicant_id.clone()),
        |result, e| match e {
            CoreCustomerEvent::CustomerKycUpdated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, updated_customer.id);
    assert_eq!(recorded.kyc_verification, KycVerification::Rejected);

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
    let (customers, outbox) = setup().await?;

    // First create a customer
    let original_email = format!("test-{}@example.com", Uuid::new_v4());
    let telegram_id = format!("telegram-{}", Uuid::new_v4());
    let customer = customers
        .create(
            &DummySubject,
            original_email,
            telegram_id,
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
