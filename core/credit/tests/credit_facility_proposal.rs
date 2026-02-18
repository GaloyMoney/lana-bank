mod helpers;

use authz::dummy::DummySubject;
use core_credit::*;
use helpers::event::expect_event;
use money::UsdCents;

/// `FacilityProposalCreated` is published when a new credit facility proposal
/// is created via `CoreCredit::create_facility_proposal()`.
///
/// # Trigger
/// `CoreCredit::create_facility_proposal`
///
/// # Consumers
/// - `History::process_credit_event` - records to credit facility history
/// - `RepaymentPlan::process_credit_event` - initializes repayment plan tracking
/// - Dashboard values
///
/// # Event Contents
/// - `id`: Unique proposal identifier
/// - `status`: Proposal status (PendingCustomerApproval at creation)
/// - `amount`: Requested facility amount
/// - `terms`: Facility terms (rate, CVL thresholds, duration, etc.)
/// - `customer_id`: Customer who requested the facility
/// - `created_at`: Timestamp of proposal creation
#[tokio::test]
async fn facility_proposal_created_event_on_create() -> anyhow::Result<()> {
    let ctx = helpers::setup().await?;

    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("test-{}@example.com", uuid::Uuid::new_v4()),
            format!("telegram-{}", uuid::Uuid::new_v4()),
            core_customer::CustomerType::Individual,
        )
        .await?;

    let deposit_account_id = CalaAccountId::new();
    let amount = UsdCents::from(1_000_000);
    let terms = helpers::test_terms();

    let (proposal, recorded) = expect_event(
        &ctx.outbox,
        || {
            ctx.credit.create_facility_proposal(
                &DummySubject,
                customer.id,
                deposit_account_id,
                amount,
                terms,
                None::<core_custody::CustodianId>,
            )
        },
        |result, e| match e {
            CoreCreditEvent::FacilityProposalCreated { entity } if entity.id == result.id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, proposal.id);
    assert_eq!(recorded.customer_id, customer.id);
    assert_eq!(recorded.amount, amount);
    assert_eq!(recorded.terms, terms);
    assert_eq!(
        recorded.status,
        CreditFacilityProposalStatus::PendingCustomerApproval
    );

    Ok(())
}

/// `FacilityProposalConcluded` is published when a credit facility proposal's
/// approval process concludes (approved or denied) via governance.
///
/// # Trigger
/// `CreditFacilityProposals::conclude_customer_approval`
/// (followed by governance approval + proposal transition jobs)
///
/// # Consumers
/// - `History::process_credit_event` - records to credit facility history
/// - `RepaymentPlan::process_credit_event` - updates repayment plan tracking
/// - Admin GraphQL subscription `creditFacilityProposalConcluded`
/// - Dashboard values
/// - Sim-bootstrap scenarios - wait for approval before collateralization
///
/// # Event Contents
/// - `id`: Unique proposal identifier
/// - `status`: Final proposal status (Approved or Denied)
/// - `amount`: Requested facility amount
/// - `terms`: Facility terms
/// - `customer_id`: Customer who requested the facility
/// - `created_at`: Timestamp of proposal creation
#[tokio::test]
#[serial_test::file_serial(core_credit_shared_jobs)]
async fn facility_proposal_concluded_event_on_approval() -> anyhow::Result<()> {
    let mut ctx = helpers::setup().await?;
    ctx.jobs.start_poll().await?;

    let customer = ctx
        .customers
        .create_customer_bypassing_kyc(
            &DummySubject,
            format!("test-{}@example.com", uuid::Uuid::new_v4()),
            format!("telegram-{}", uuid::Uuid::new_v4()),
            core_customer::CustomerType::Individual,
        )
        .await?;

    let deposit_account_id = CalaAccountId::new();
    let amount = UsdCents::from(1_000_000);
    let terms = helpers::test_terms();

    let proposal = ctx
        .credit
        .create_facility_proposal(
            &DummySubject,
            customer.id,
            deposit_account_id,
            amount,
            terms,
            None::<core_custody::CustodianId>,
        )
        .await?;

    let proposal_id = proposal.id;
    let proposals = ctx.credit.proposals().clone();
    let (_, recorded) = expect_event(
        &ctx.outbox,
        move || {
            let proposals = proposals.clone();
            async move {
                proposals
                    .conclude_customer_approval(&DummySubject, proposal_id, true)
                    .await
            }
        },
        |_result, e| match e {
            CoreCreditEvent::FacilityProposalConcluded { entity } if entity.id == proposal_id => {
                Some(entity.clone())
            }
            _ => None,
        },
    )
    .await?;

    assert_eq!(recorded.id, proposal.id);
    assert_eq!(recorded.customer_id, customer.id);
    assert_eq!(recorded.amount, amount);
    assert_eq!(recorded.terms, terms);
    assert_eq!(recorded.status, CreditFacilityProposalStatus::Approved);
    ctx.jobs.shutdown().await?;

    Ok(())
}
