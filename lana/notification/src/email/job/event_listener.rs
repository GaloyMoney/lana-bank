use tracing::{Span, instrument};

use lana_events::{
    CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent,
};
use obix::out::{OutboxEventHandler, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::deposit_account_created_email::DepositAccountCreatedEmailConfig;
use super::obligation_overdue_email::ObligationOverdueEmailConfig;
use super::partial_liquidation_email::PartialLiquidationEmailConfig;
use super::role_created_email::RoleCreatedEmailConfig;
use super::under_margin_call_email::UnderMarginCallEmailConfig;

pub const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");

pub struct EmailEventListenerHandler {
    obligation_overdue: JobSpawner<ObligationOverdueEmailConfig>,
    partial_liquidation: JobSpawner<PartialLiquidationEmailConfig>,
    under_margin_call: JobSpawner<UnderMarginCallEmailConfig>,
    deposit_account_created: JobSpawner<DepositAccountCreatedEmailConfig>,
    role_created: JobSpawner<RoleCreatedEmailConfig>,
}

impl EmailEventListenerHandler {
    pub fn new(
        obligation_overdue: JobSpawner<ObligationOverdueEmailConfig>,
        partial_liquidation: JobSpawner<PartialLiquidationEmailConfig>,
        under_margin_call: JobSpawner<UnderMarginCallEmailConfig>,
        deposit_account_created: JobSpawner<DepositAccountCreatedEmailConfig>,
        role_created: JobSpawner<RoleCreatedEmailConfig>,
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

impl OutboxEventHandler<LanaEvent> for EmailEventListenerHandler {
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
                self.obligation_overdue
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        ObligationOverdueEmailConfig {
                            obligation_id: entity.id,
                            credit_facility_id,
                            outstanding_amount: entity.outstanding_amount,
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
                    .ok_or("liquidation_trigger must be set for PartialLiquidationInitiated")?;
                self.partial_liquidation
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        PartialLiquidationEmailConfig {
                            credit_facility_id: entity.id,
                            customer_id: entity.customer_id,
                            trigger_price: trigger.trigger_price,
                            initially_estimated_to_liquidate: trigger
                                .initially_estimated_to_liquidate,
                            initially_expected_to_receive: trigger.initially_expected_to_receive,
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
                let effective = event.recorded_at.date_naive();
                self.under_margin_call
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        UnderMarginCallEmailConfig {
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
            Some(LanaEvent::Deposit(
                deposit_event @ CoreDepositEvent::DepositAccountCreated { entity },
            )) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", deposit_event.as_ref());

                self.deposit_account_created
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        DepositAccountCreatedEmailConfig {
                            account_id: entity.id,
                            account_holder_id: entity.account_holder_id,
                        },
                        entity.id.to_string(),
                    )
                    .await?;
            }
            Some(LanaEvent::CoreAccess(access_event @ CoreAccessEvent::RoleCreated { entity })) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", access_event.as_ref());

                self.role_created
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        RoleCreatedEmailConfig {
                            role_id: entity.id,
                            role_name: entity.name.clone(),
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
