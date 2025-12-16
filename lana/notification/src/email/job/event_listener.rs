use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use lana_events::{CoreAccessEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent};
use outbox::{Outbox, PersistentOutboxEvent};

use crate::email::EmailNotification;

#[derive(Serialize, Deserialize)]
pub struct EmailEventListenerConfig<AuthzType>(std::marker::PhantomData<AuthzType>);

impl<AuthzType> Default for EmailEventListenerConfig<AuthzType> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<AuthzType> JobConfig for EmailEventListenerConfig<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    type Initializer = EmailEventListenerInit<AuthzType>;
}

pub struct EmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<AuthzType>,
}

impl<AuthzType> EmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    pub fn new(
        outbox: &Outbox<LanaEvent>,
        email_notification: &EmailNotification<AuthzType>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            email_notification: email_notification.clone(),
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");
impl<AuthzType> JobInitializer for EmailEventListenerInit<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailEventListenerRunner {
            outbox: self.outbox.clone(),
            email_notification: self.email_notification.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct EmailEventListenerJobData {
    sequence: outbox::EventSequence,
}

pub struct EmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck,
{
    outbox: Outbox<LanaEvent>,
    email_notification: EmailNotification<AuthzType>,
}

impl<AuthzType> EmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    #[instrument(name = "notification.email_listener_job.process_message", parent = None, skip(self, message, op), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<LanaEvent>,
        op: &mut impl es_entity::AtomicOperation,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::ObligationOverdue {
                    id,
                    credit_facility_id,
                    amount,
                },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                self.email_notification
                    .send_obligation_overdue_notification(op, id, credit_facility_id, amount)
                    .await?;
            }
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::PartialLiquidationInitiated {
                    credit_facility_id,
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
                    .send_partial_liquidation_initiated_notification(
                        op,
                        credit_facility_id,
                        trigger_price,
                        initially_estimated_to_liquidate,
                        initially_expected_to_receive,
                    )
                    .await?;
            }
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::FacilityCollateralizationChanged {
                    id,
                    state: core_credit::CollateralizationState::UnderMarginCallThreshold,
                    recorded_at,
                    effective,
                    collateral,
                    outstanding,
                    price,
                },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                self.email_notification
                    .send_under_margin_call_notification(
                        op,
                        id,
                        recorded_at,
                        effective,
                        collateral,
                        &outstanding.disbursed,
                        &outstanding.interest,
                        price,
                    )
                    .await?;
            }
            Some(LanaEvent::Deposit(
                deposit_event @ CoreDepositEvent::DepositAccountCreated {
                    id,
                    account_holder_id,
                },
            )) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", deposit_event.as_ref());

                self.email_notification
                    .send_deposit_account_created_notification(op, id, account_holder_id)
                    .await?;
            }
            Some(LanaEvent::Access(access_event @ CoreAccessEvent::RoleCreated { id, name })) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", access_event.as_ref());

                self.email_notification
                    .send_role_created_notification(op, id, name)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<AuthzType> JobRunner for EmailEventListenerRunner<AuthzType>
where
    AuthzType: authz::PermissionCheck + Clone + Send + Sync + 'static,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action: From<core_credit::CoreCreditAction>
        + From<core_customer::CoreCustomerAction>
        + From<core_access::CoreAccessAction>
        + From<core_deposit::CoreDepositAction>
        + From<governance::GovernanceAction>
        + From<core_custody::CoreCustodyAction>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object: From<core_credit::CoreCreditObject>
        + From<core_customer::CustomerObject>
        + From<core_access::CoreAccessObject>
        + From<core_deposit::CoreDepositObject>
        + From<governance::GovernanceObject>
        + From<core_custody::CoreCustodyObject>,
    <<AuthzType as authz::PermissionCheck>::Audit as audit::AuditSvc>::Subject:
        From<core_access::UserId>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailEventListenerJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

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
                            self.process_message(message.as_ref(), &mut op).await?;
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
