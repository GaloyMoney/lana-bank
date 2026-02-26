use tracing::{Span, instrument};

use lana_events::{
    CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent,
};
use obix::out::{OutboxEventHandler, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::process_deposit_account_created_notification::DepositAccountCreatedNotificationConfig;
use super::process_margin_call_notification::MarginCallNotificationConfig;
use super::process_obligation_overdue_notification::ObligationOverdueNotificationConfig;
use super::process_partial_liquidation_notification::PartialLiquidationNotificationConfig;
use super::process_role_created_notification::RoleCreatedNotificationConfig;

pub const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");

pub struct EmailEventListenerHandler {
    obligation_overdue_notification: JobSpawner<ObligationOverdueNotificationConfig>,
    partial_liquidation_notification: JobSpawner<PartialLiquidationNotificationConfig>,
    margin_call_notification: JobSpawner<MarginCallNotificationConfig>,
    deposit_account_created_notification: JobSpawner<DepositAccountCreatedNotificationConfig>,
    role_created_notification: JobSpawner<RoleCreatedNotificationConfig>,
}

impl EmailEventListenerHandler {
    pub fn new(
        obligation_overdue_notification: JobSpawner<ObligationOverdueNotificationConfig>,
        partial_liquidation_notification: JobSpawner<PartialLiquidationNotificationConfig>,
        margin_call_notification: JobSpawner<MarginCallNotificationConfig>,
        deposit_account_created_notification: JobSpawner<DepositAccountCreatedNotificationConfig>,
        role_created_notification: JobSpawner<RoleCreatedNotificationConfig>,
    ) -> Self {
        Self {
            obligation_overdue_notification,
            partial_liquidation_notification,
            margin_call_notification,
            deposit_account_created_notification,
            role_created_notification,
        }
    }
}

impl OutboxEventHandler<LanaEvent> for EmailEventListenerHandler {
    #[instrument(name = "notification.email_listener_job.process_message_in_op", parent = None, skip_all, fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
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
                self.obligation_overdue_notification
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ObligationOverdueNotificationConfig {
                            obligation_id: entity.id,
                            credit_facility_id,
                            outstanding_amount: entity.outstanding_amount,
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
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
                self.partial_liquidation_notification
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        PartialLiquidationNotificationConfig {
                            credit_facility_id: entity.id,
                            customer_id: entity.customer_id,
                            trigger_price: trigger.trigger_price,
                            initially_estimated_to_liquidate: trigger
                                .initially_estimated_to_liquidate,
                            initially_expected_to_receive: trigger.initially_expected_to_receive,
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
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
                self.margin_call_notification
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        MarginCallNotificationConfig {
                            credit_facility_id: entity.id,
                            customer_id: entity.customer_id,
                            effective: event.recorded_at.date_naive(),
                            collateral: collateralization.collateral,
                            outstanding_disbursed: collateralization.outstanding.disbursed,
                            outstanding_interest: collateralization.outstanding.interest,
                            price: collateralization.price_at_state_change,
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            Some(LanaEvent::Deposit(
                deposit_event @ CoreDepositEvent::DepositAccountCreated { entity },
            )) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", deposit_event.as_ref());

                self.deposit_account_created_notification
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        DepositAccountCreatedNotificationConfig {
                            account_id: entity.id,
                            account_holder_id: entity.account_holder_id,
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            Some(LanaEvent::CoreAccess(access_event @ CoreAccessEvent::RoleCreated { entity })) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", access_event.as_ref());

                self.role_created_notification
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        RoleCreatedNotificationConfig {
                            role_id: entity.id,
                            role_name: entity.name.clone(),
                            trace_context: Some(tracing_utils::persistence::extract()),
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
