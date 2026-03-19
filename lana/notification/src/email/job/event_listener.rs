use tracing::{Span, instrument};

use lana_events::{CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::send_deposit_account_created_email::SendDepositAccountCreatedEmailConfig;
use super::send_obligation_overdue_email::SendObligationOverdueEmailConfig;
use super::send_partial_liquidation_email::SendPartialLiquidationEmailConfig;
use super::send_role_created_email::SendRoleCreatedEmailConfig;
use super::send_under_margin_call_email::SendUnderMarginCallEmailConfig;

pub const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");

pub struct EmailEventListenerHandler {
    obligation_overdue: JobSpawner<SendObligationOverdueEmailConfig>,
    partial_liquidation: JobSpawner<SendPartialLiquidationEmailConfig>,
    under_margin_call: JobSpawner<SendUnderMarginCallEmailConfig>,
    deposit_account_created: JobSpawner<SendDepositAccountCreatedEmailConfig>,
    role_created: JobSpawner<SendRoleCreatedEmailConfig>,
}

impl EmailEventListenerHandler {
    pub fn new(
        obligation_overdue: JobSpawner<SendObligationOverdueEmailConfig>,
        partial_liquidation: JobSpawner<SendPartialLiquidationEmailConfig>,
        under_margin_call: JobSpawner<SendUnderMarginCallEmailConfig>,
        deposit_account_created: JobSpawner<SendDepositAccountCreatedEmailConfig>,
        role_created: JobSpawner<SendRoleCreatedEmailConfig>,
    ) -> Self {
        Self {
            obligation_overdue,
            partial_liquidation,
            under_margin_call,
            deposit_account_created,
            role_created,
        }
    }
}

impl<E> OutboxEventHandler<E> for EmailEventListenerHandler
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreAccessEvent>,
{
    #[instrument(name = "notification.email_listener_job.process_message_in_op", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(credit_event @ CoreCreditCollectionEvent::ObligationOverdue { entity }) =
            event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", credit_event.as_ref());

            let credit_facility_id: core_credit::CreditFacilityId = entity.beneficiary_id.into();
            self.obligation_overdue
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendObligationOverdueEmailConfig {
                        obligation_id: entity.id,
                        credit_facility_id,
                        outstanding_amount: entity.outstanding_amount,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        if let Some(credit_event @ CoreCreditEvent::PartialLiquidationInitiated { entity }) =
            event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", credit_event.as_ref());

            let trigger = entity
                .liquidation_trigger
                .as_ref()
                .ok_or("liquidation_trigger must be set for PartialLiquidationInitiated")?;
            self.partial_liquidation
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendPartialLiquidationEmailConfig {
                        credit_facility_id: entity.id,
                        customer_id: entity.customer_id,
                        trigger_price: trigger.trigger_price,
                        initially_estimated_to_liquidate: trigger.initially_estimated_to_liquidate,
                        initially_expected_to_receive: trigger.initially_expected_to_receive,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        if let Some(credit_event @ CoreCreditEvent::FacilityCollateralizationChanged { entity }) =
            event.as_event()
            && entity.collateralization.state
                == core_credit::CollateralizationState::UnderMarginCallThreshold
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", credit_event.as_ref());

            let collateralization = &entity.collateralization;
            let effective = event.recorded_at.date_naive();
            self.under_margin_call
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendUnderMarginCallEmailConfig {
                        credit_facility_id: entity.id,
                        customer_id: entity.customer_id,
                        effective_date: effective,
                        collateral: collateralization.collateral,
                        outstanding_disbursed: collateralization.outstanding.disbursed,
                        outstanding_interest: collateralization.outstanding.interest,
                        price: collateralization.price_at_state_change,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        if let Some(deposit_event @ CoreDepositEvent::DepositAccountCreated { entity }) =
            event.as_event()
        {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", deposit_event.as_ref());

            self.deposit_account_created
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendDepositAccountCreatedEmailConfig {
                        account_id: entity.id,
                        account_holder_id: entity.account_holder_id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        if let Some(access_event @ CoreAccessEvent::RoleCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", access_event.as_ref());

            self.role_created
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendRoleCreatedEmailConfig {
                        role_id: entity.id,
                        role_name: entity.name.clone(),
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
