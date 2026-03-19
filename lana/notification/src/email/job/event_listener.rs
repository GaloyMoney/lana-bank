use tracing::{Span, instrument};

use authz::PermissionCheck;
use core_access::user::{Users, UsersByCreatedAtCursor};
use core_customer::Customers;
use lana_events::{
    CoreAccessEvent, CoreCreditCollectionEvent, CoreCreditEvent, CoreDepositEvent, LanaEvent,
};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::{JobId, JobSpawner, JobType};

use super::send_deposit_account_created_email::SendDepositAccountCreatedEmailConfig;
use super::send_obligation_overdue_email::SendObligationOverdueEmailConfig;
use super::send_partial_liquidation_email::SendPartialLiquidationEmailConfig;
use super::send_role_created_email::SendRoleCreatedEmailConfig;
use super::send_under_margin_call_email::SendUnderMarginCallEmailConfig;

pub const EMAIL_LISTENER_JOB: JobType = JobType::new("outbox.email-listener");

pub struct EmailEventListenerHandler<Perms>
where
    Perms: PermissionCheck,
{
    users: Users<Perms::Audit, LanaEvent>,
    customers: Customers<Perms, LanaEvent>,
    send_obligation_overdue_email: JobSpawner<SendObligationOverdueEmailConfig>,
    send_partial_liquidation_email: JobSpawner<SendPartialLiquidationEmailConfig>,
    send_under_margin_call_email: JobSpawner<SendUnderMarginCallEmailConfig>,
    send_deposit_account_created_email: JobSpawner<SendDepositAccountCreatedEmailConfig>,
    send_role_created_email: JobSpawner<SendRoleCreatedEmailConfig>,
}

impl<Perms> EmailEventListenerHandler<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        users: Users<Perms::Audit, LanaEvent>,
        customers: Customers<Perms, LanaEvent>,
        send_obligation_overdue_email: JobSpawner<SendObligationOverdueEmailConfig>,
        send_partial_liquidation_email: JobSpawner<SendPartialLiquidationEmailConfig>,
        send_under_margin_call_email: JobSpawner<SendUnderMarginCallEmailConfig>,
        send_deposit_account_created_email: JobSpawner<SendDepositAccountCreatedEmailConfig>,
        send_role_created_email: JobSpawner<SendRoleCreatedEmailConfig>,
    ) -> Self {
        Self {
            users,
            customers,
            send_obligation_overdue_email,
            send_partial_liquidation_email,
            send_under_margin_call_email,
            send_deposit_account_created_email,
            send_role_created_email,
        }
    }
}

impl<Perms> EmailEventListenerHandler<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Subject: From<core_access::UserId>,
{
    async fn list_all_user_emails(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut emails = Vec::new();
        let mut has_next_page = true;
        let mut after = None;
        while has_next_page {
            let es_entity::PaginatedQueryRet {
                entities,
                has_next_page: next_page,
                end_cursor,
            } = self
                .users
                .list_users_without_audit(
                    es_entity::PaginatedQueryArgs::<UsersByCreatedAtCursor> { first: 20, after },
                    es_entity::ListDirection::Descending,
                )
                .await?;
            (after, has_next_page) = (end_cursor, next_page);

            for user in entities {
                emails.push(user.email);
            }
        }
        Ok(emails)
    }
}

impl<Perms, E> OutboxEventHandler<E> for EmailEventListenerHandler<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_access::CoreAccessAction> + From<core_customer::CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_access::CoreAccessObject> + From<core_customer::CustomerObject>,
    <<Perms as PermissionCheck>::Audit as audit::AuditSvc>::Subject: From<core_access::UserId>,
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
            let user_emails = self.list_all_user_emails().await?;
            for email in user_emails {
                self.send_obligation_overdue_email
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        SendObligationOverdueEmailConfig {
                            obligation_id: entity.id,
                            credit_facility_id,
                            outstanding_amount: entity.outstanding_amount,
                            recipient_email: email,
                        },
                        format!("{}:{}", entity.id, email),
                    )
                    .await?;
            }
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

            let user_emails = self.list_all_user_emails().await?;
            for email in user_emails {
                self.send_partial_liquidation_email
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        SendPartialLiquidationEmailConfig {
                            credit_facility_id: entity.id,
                            customer_id: entity.customer_id,
                            trigger_price: trigger.trigger_price,
                            initially_estimated_to_liquidate: trigger
                                .initially_estimated_to_liquidate,
                            initially_expected_to_receive: trigger.initially_expected_to_receive,
                            recipient_email: email,
                        },
                        format!("{}:{}", entity.id, email),
                    )
                    .await?;
            }

            let party = self
                .customers
                .find_party_by_customer_id_without_audit(entity.customer_id)
                .await?;
            self.send_partial_liquidation_email
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    SendPartialLiquidationEmailConfig {
                        credit_facility_id: entity.id,
                        customer_id: entity.customer_id,
                        trigger_price: trigger.trigger_price,
                        initially_estimated_to_liquidate: trigger.initially_estimated_to_liquidate,
                        initially_expected_to_receive: trigger.initially_expected_to_receive,
                        recipient_email: party.email.clone(),
                    },
                    format!("{}:{}", entity.id, party.email),
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
            self.send_under_margin_call_email
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

            self.send_deposit_account_created_email
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

            let user_emails = self.list_all_user_emails().await?;
            for email in user_emails {
                self.send_role_created_email
                    .spawn_with_queue_id_in_op(
                        op,
                        JobId::new(),
                        SendRoleCreatedEmailConfig {
                            role_id: entity.id,
                            role_name: entity.name.clone(),
                            recipient_email: email,
                        },
                        format!("{}:{}", entity.id, email),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
