use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    DepositId, GovernanceAction, GovernanceObject, UsdCents, WithdrawalId,
};
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ExportToSumsubConfig {
    Deposit {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    Withdrawal {
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
}

pub const EXPORT_TO_SUMSUB_COMMAND: JobType = JobType::new("command.deposit-sync.export-to-sumsub");

pub struct ExportToSumsubJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    sumsub_client: sumsub::SumsubClient,
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> ExportToSumsubJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    pub fn new(
        sumsub_client: sumsub::SumsubClient,
        deposits: CoreDeposit<Perms, E>,
        customers: Customers<Perms, E>,
    ) -> Self {
        Self {
            sumsub_client,
            deposits,
            customers,
        }
    }
}

impl<Perms, E> JobInitializer for ExportToSumsubJobInitializer<Perms, E>
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
    type Config = ExportToSumsubConfig;

    fn job_type(&self) -> JobType {
        EXPORT_TO_SUMSUB_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExportToSumsubJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
            deposits: self.deposits.clone(),
            customers: self.customers.clone(),
        }))
    }
}

pub struct ExportToSumsubJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<LanaEvent>
        + std::fmt::Debug,
{
    config: ExportToSumsubConfig,
    sumsub_client: sumsub::SumsubClient,
    deposits: CoreDeposit<Perms, E>,
    customers: Customers<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExportToSumsubJobRunner<Perms, E>
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
        name = "deposit_sync.export_to_sumsub_job.process_command",
        skip(self, _current_job)
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let (deposit_account_id, tx_id, tx_type, direction, amount) = match &self.config {
            ExportToSumsubConfig::Deposit {
                id,
                deposit_account_id,
                amount,
            } => (
                *deposit_account_id,
                id.to_string(),
                "Deposit",
                "in",
                *amount,
            ),
            ExportToSumsubConfig::Withdrawal {
                id,
                deposit_account_id,
                amount,
            } => (
                *deposit_account_id,
                id.to_string(),
                "Withdrawal",
                "out",
                *amount,
            ),
        };

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
                    tx_id,
                    tx_type,
                    direction,
                    amount_usd,
                    "USD",
                )
                .await?;
        } else {
            tracing::warn!(
                tx_type = tx_type,
                customer_id = %account.account_holder_id,
                kyc_level = ?customer.level,
                "Skipping sync for non verified customer transaction"
            );
        }

        Ok(JobCompletion::Complete)
    }
}
