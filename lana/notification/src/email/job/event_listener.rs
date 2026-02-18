use tracing::{Span, instrument};

use lana_events::{
    CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent,
};
use obix::out::{OutboxEventHandler, PersistentOutboxEvent};

use job::JobType;

use crate::email::EmailNotification;

pub const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");

pub struct EmailEventListenerHandler<Perms>
where
    Perms: authz::PermissionCheck,
{
    email_notification: EmailNotification<Perms>,
}

impl<Perms> EmailEventListenerHandler<Perms>
where
    Perms: authz::PermissionCheck,
{
    pub fn new(email_notification: &EmailNotification<Perms>) -> Self {
        Self {
            email_notification: email_notification.clone(),
        }
    }
}

impl<Perms> OutboxEventHandler<LanaEvent> for EmailEventListenerHandler<Perms>
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
    #[instrument(name = "notification.email_listener_job.process_message_in_op", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<LanaEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(LanaEvent::CreditCollection(
                credit_event @ CoreCreditCollectionEvent::ObligationOverdue { entity },
            )) => {
                event.inject_trace_parent();
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
                credit_event @ CoreCreditEvent::PartialLiquidationInitiated { entity },
            )) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                let trigger = entity
                    .liquidation_trigger
                    .as_ref()
                    .expect("liquidation_trigger must be set for PartialLiquidationInitiated");
                self.email_notification
                    .send_partial_liquidation_initiated_notification_in_op(
                        op,
                        &entity.id,
                        &entity.customer_id,
                        &trigger.trigger_price,
                        &trigger.initially_estimated_to_liquidate,
                        &trigger.initially_expected_to_receive,
                    )
                    .await?;
            }
            Some(LanaEvent::Credit(
                credit_event @ CoreCreditEvent::FacilityCollateralizationChanged { entity },
            )) if entity.collateralization.state
                == core_credit::CollateralizationState::UnderMarginCallThreshold =>
            {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", credit_event.as_ref());

                let collateralization = &entity.collateralization;
                let effective = event.recorded_at.date_naive();
                self.email_notification
                    .send_under_margin_call_notification_in_op(
                        op,
                        &entity.id,
                        &entity.customer_id,
                        &effective,
                        &collateralization.collateral,
                        &collateralization.outstanding.disbursed,
                        &collateralization.outstanding.interest,
                        &collateralization.price_at_state_change,
                    )
                    .await?;
            }
            Some(LanaEvent::Deposit(
                deposit_event @ CoreDepositEvent::DepositAccountCreated { entity },
            )) => {
                event.inject_trace_parent();
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
                event.inject_trace_parent();
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
