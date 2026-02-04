use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use job::*;
use lana_events::{
    CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent,
};
use obix::out::{Outbox, PersistentOutboxEvent};

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize)]
pub struct EmailEventListenerConfig<Perms>(std::marker::PhantomData<Perms>);

impl<Perms> Default for EmailEventListenerConfig<Perms> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub struct EmailEventListenerInit<Perms>
where
    Perms: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<Perms>,
}

impl<Perms> EmailEventListenerInit<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub fn new(outbox: &Outbox<LanaEvent>, email_notification: &EmailNotification<Perms>) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");
impl<Perms> JobInitializer for EmailEventListenerInit<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    type Config = EmailEventListenerConfig<Perms>;
    fn job_type(&self) -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailEventListenerRunner {
            outbox: self.outbox.clone(),
            email_notification: self.email_notification.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct EmailEventListenerJobData {
    sequence: obix::EventSequence,
}

pub struct EmailEventListenerRunner<Perms>
where
    Perms: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<Perms>,
}

impl<Perms> EmailEventListenerRunner<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[instrument(name = "notification.email_listener_job.process_message_in_op", parent = None, skip(self, op, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        message: &PersistentOutboxEvent<LanaEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(LanaEvent::CreditCollection(
                credit_event @ CoreCreditCollectionEvent::ObligationOverdue { entity },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                let credit_facility_id: core_credit::CreditFacilityId =
                    entity.beneficiary_id.into();
                self.email_notification
                    .send_obligation_overdue_notification_in_op(
                        op,
                        &entity.id,
                        &credit_facility_id,
                        &entity.outstanding_amount,
                    )
                    .await?;
            }
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::PartialLiquidationInitiated {
                    credit_facility_id,
                    customer_id,
                    trigger_price,
                    initially_expected_to_receive,
                    initially_estimated_to_liquidate,
                    ..
                },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                self.email_notification
                    .send_partial_liquidation_initiated_notification_in_op(
                        op,
                        credit_facility_id,
                        customer_id,
                        trigger_price,
                        initially_estimated_to_liquidate,
                        initially_expected_to_receive,
                    )
                    .await?;
            }
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::FacilityCollateralizationChanged {
                    id,
                    customer_id,
                    state: core_credit::CollateralizationState::UnderMarginCallThreshold,
                    effective,
                    collateral,
                    outstanding,
                    price,
                    ..
                },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                self.email_notification
                    .send_under_margin_call_notification_in_op(
                        op,
                        id,
                        customer_id,
                        effective,
                        collateral,
                        &outstanding.disbursed,
                        &outstanding.interest,
                        price,
                    )
                    .await?;
            }
            Some(LanaEvent::Deposit(
                deposit_event @ CoreDepositEvent::DepositAccountCreated { entity },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", deposit_event.as_ref());

                self.email_notification
                    .send_deposit_account_created_notification_in_op(
                        op,
                        &entity.id,
                        &entity.account_holder_id,
                    )
                    .await?;
            }
            Some(LanaEvent::CoreAccess(access_event @ CoreAccessEvent::RoleCreated { entity })) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", access_event.as_ref());

                self.email_notification
                    .send_role_created_notification_in_op(op, &entity.id, &entity.name)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms> JobRunner for EmailEventListenerRunner<Perms>
where
    Perms: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailEventListenerJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %EMAIL_LISTENER_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut op = current_job.pool().begin().await?;
                            self.process_message_in_op(&mut op, message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut op, &state)
                                .await?;
                            op.commit().await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
