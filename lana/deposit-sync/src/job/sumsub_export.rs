use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    DepositId, GovernanceAction, GovernanceObject, UsdCents, WithdrawalId,
};
use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use sumsub::SumsubClient;

use job::JobType;
use lana_events::LanaEvent;

pub const SUMSUB_EXPORT_JOB: JobType = JobType::new("outbox.sumsub-export");

pub struct SumsubExportHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    sumsub_client: SumsubClient,
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> SumsubExportHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    pub fn new(
        sumsub_client: SumsubClient,
        deposits: &CoreDeposit<Perms, E>,
        customers: &Customers<Perms, E>,
    ) -> Self {
        Self {
            sumsub_client,
            deposits: deposits.clone(),
            customers: customers.clone(),
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for SumsubExportHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    #[instrument(name = "deposit_sync.sumsub_export_job.process_message", parent = None, skip(self, _op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        _op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.as_event() {
            Some(e @ CoreDepositEvent::DepositInitialized { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                self.handle_deposit(entity.id, entity.deposit_account_id, entity.amount)
                    .await?;
            }
            Some(e @ CoreDepositEvent::WithdrawalConfirmed { entity }) => {
                event.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", e.as_ref());

                self.handle_withdrawal(entity.id, entity.deposit_account_id, entity.amount)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl<Perms, E> SumsubExportHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    async fn handle_deposit(
        &self,
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let account = self
            .deposits
            .find_account_by_id_without_audit(deposit_account_id)
            .await?;

        let customer = self
            .customers
            .find_by_id_without_audit(account.account_holder_id)
            .await?;

        if customer.should_sync_financial_transactions() {
            let amount_usd: f64 = amount.to_usd().try_into()?;
            self.sumsub_client
                .submit_finance_transaction(
                    account.account_holder_id,
                    id.to_string(),
                    "Deposit",
                    "in",
                    amount_usd,
                    "USD",
                )
                .await?;
        } else {
            tracing::warn!(
                deposit_id = %id,
                customer_id = %account.account_holder_id,
                kyc_level = ?customer.level,
                "Skipping sync for non verified customer deposit"
            );
        }
        Ok(())
    }

    async fn handle_withdrawal(
        &self,
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let account = self
            .deposits
            .find_account_by_id_without_audit(deposit_account_id)
            .await?;

        let customer = self
            .customers
            .find_by_id_without_audit(account.account_holder_id)
            .await?;

        if customer.should_sync_financial_transactions() {
            let amount_usd: f64 = amount.to_usd().try_into()?;
            self.sumsub_client
                .submit_finance_transaction(
                    account.account_holder_id,
                    id.to_string(),
                    "Withdrawal",
                    "out",
                    amount_usd,
                    "USD",
                )
                .await?;
        } else {
            tracing::warn!(
                withdrawal_id = %id,
                customer_id = %account.account_holder_id,
                kyc_level = ?customer.level,
                "Skipping sync for non verified customer withdrawal"
            );
        }
        Ok(())
    }
}
