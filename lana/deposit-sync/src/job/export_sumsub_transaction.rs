use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    GovernanceAction, GovernanceObject, UsdCents,
};
use governance::GovernanceEvent;
use job::*;
use lana_events::LanaEvent;
use obix::out::OutboxEventMarker;
use sumsub::SumsubClient;
use tracing_macros::record_error_severity;

pub const EXPORT_SUMSUB_TRANSACTION_COMMAND: JobType =
    JobType::new("command.deposit-sync.export-sumsub-transaction");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportSumsubTransactionConfig {
    pub transaction_type: String,
    pub direction: String,
    pub transaction_id: String,
    pub deposit_account_id: DepositAccountId,
    pub amount: UsdCents,
}

pub struct ExportSumsubTransactionJobInitializer<Perms, E>
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

impl<Perms, E> ExportSumsubTransactionJobInitializer<Perms, E>
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

impl<Perms, E> JobInitializer for ExportSumsubTransactionJobInitializer<Perms, E>
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
    type Config = ExportSumsubTransactionConfig;

    fn job_type(&self) -> JobType {
        EXPORT_SUMSUB_TRANSACTION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExportSumsubTransactionJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
            deposits: self.deposits.clone(),
            customers: self.customers.clone(),
        }))
    }
}

struct ExportSumsubTransactionJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    config: ExportSumsubTransactionConfig,
    sumsub_client: SumsubClient,
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExportSumsubTransactionJobRunner<Perms, E>
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
    #[record_error_severity]
    #[tracing::instrument(
        name = "deposit_sync.export_sumsub_transaction.process_command",
        skip_all
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let account = self
            .deposits
            .find_account_by_id_without_audit(self.config.deposit_account_id)
            .await?;

        let customer = self
            .customers
            .find_by_id_without_audit(account.account_holder_id)
            .await?;

        // Valid use case branching
        // lint:allow(service-conditionals)
        if customer.should_sync_financial_transactions() {
            let amount_usd: f64 = self.config.amount.to_usd().try_into()?;
            self.sumsub_client
                .submit_finance_transaction(
                    account.account_holder_id,
                    self.config.transaction_id.clone(),
                    &self.config.transaction_type,
                    &self.config.direction,
                    amount_usd,
                    "USD",
                )
                .await?;
        } else {
            tracing::warn!(
                transaction_id = %self.config.transaction_id,
                customer_id = %account.account_holder_id,
                kyc_level = ?customer.level,
                "Skipping sync for non verified customer transaction"
            );
        }
        Ok(JobCompletion::Complete)
    }
}
