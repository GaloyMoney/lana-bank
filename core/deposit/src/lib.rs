#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod account;
mod chart_of_accounts_integration;
mod deposit;
mod deposit_account_balance;
pub mod error;
mod event;
mod for_subject;
mod history;
mod ledger;
mod primitives;
mod processes;
mod publisher;
mod withdrawal;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use core_accounting::{Chart, LedgerTransactionInitiator};
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject, Customers};
use domain_config::ExposedDomainConfigsReadOnly;
use governance::{Governance, GovernanceEvent};
use job::Jobs;
use obix::out::{Outbox, OutboxEventMarker};
use public_id::PublicIds;

use account::*;
pub use account::{DepositAccount, DepositAccountsByCreatedAtCursor, error::DepositAccountError};
pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use deposit::*;
pub use deposit::{Deposit, DepositsByCreatedAtCursor};
pub use deposit_account_balance::DepositAccountBalance;
pub use domain_config::RequireVerifiedCustomerForAccount;
use error::*;
pub use event::*;
pub use for_subject::DepositsForSubject;
pub use history::{DepositAccountHistoryCursor, DepositAccountHistoryEntry};
use ledger::*;
pub use primitives::*;
pub use processes::approval::APPROVE_WITHDRAWAL_PROCESS;
use processes::approval::{ApproveWithdrawal, WithdrawApprovalInit, WithdrawApprovalJobConfig};
use publisher::DepositPublisher;
use withdrawal::*;
pub use withdrawal::{Withdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::account::DepositAccountEvent;
    pub use crate::deposit::DepositEvent;
    pub use crate::withdrawal::WithdrawalEvent;
}

pub struct CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    deposit_accounts: DepositAccountRepo<E>,
    deposits: DepositRepo<E>,
    withdrawals: WithdrawalRepo<E>,
    approve_withdrawal: ApproveWithdrawal<Perms, E>,
    ledger: DepositLedger,
    cala: CalaLedger,
    authz: Perms,
    governance: Governance<Perms, E>,
    outbox: Outbox<E>,
    public_ids: PublicIds,
    customers: Customers<Perms, E>,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms, E> Clone for CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            deposit_accounts: self.deposit_accounts.clone(),
            deposits: self.deposits.clone(),
            withdrawals: self.withdrawals.clone(),
            ledger: self.ledger.clone(),
            cala: self.cala.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
            approve_withdrawal: self.approve_withdrawal.clone(),
            outbox: self.outbox.clone(),
            public_ids: self.public_ids.clone(),
            customers: self.customers.clone(),
            domain_configs: self.domain_configs.clone(),
        }
    }
}

impl<Perms, E> CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction> + From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject> + From<CustomerObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "deposit.init", skip_all, fields(journal_id = %journal_id))]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
        governance: &Governance<Perms, E>,
        jobs: &mut Jobs,
        cala: &CalaLedger,
        journal_id: CalaJournalId,
        public_ids: &PublicIds,
        customers: &Customers<Perms, E>,
        domain_configs: &ExposedDomainConfigsReadOnly,
    ) -> Result<Self, CoreDepositError> {
        let clock = jobs.clock().clone();

        let publisher = DepositPublisher::new(outbox);
        let accounts = DepositAccountRepo::new(pool, &publisher, clock.clone());
        let deposits = DepositRepo::new(pool, &publisher, clock.clone());
        let withdrawals = WithdrawalRepo::new(pool, &publisher, clock.clone());
        let ledger = DepositLedger::init(cala, journal_id, clock.clone()).await?;

        let approve_withdrawal = ApproveWithdrawal::new(
            &withdrawals,
            authz.audit(),
            governance,
            &ledger,
            clock.clone(),
        );

        let approve_withdrawal_job_spawner =
            jobs.add_initializer(WithdrawApprovalInit::new(outbox, &approve_withdrawal));

        approve_withdrawal_job_spawner
            .spawn_unique(
                job::JobId::new(),
                WithdrawApprovalJobConfig::<Perms, E>::new(),
            )
            .await?;

        match governance.init_policy(APPROVE_WITHDRAWAL_PROCESS).await {
            Err(governance::error::GovernanceError::PolicyError(
                governance::policy_error::PolicyError::DuplicateApprovalProcessType,
            )) => (),
            Err(e) => return Err(e.into()),
            _ => (),
        }

        let res = Self {
            deposit_accounts: accounts,
            deposits,
            withdrawals,
            authz: authz.clone(),
            outbox: outbox.clone(),
            governance: governance.clone(),
            cala: cala.clone(),
            approve_withdrawal,
            ledger,
            public_ids: public_ids.clone(),
            customers: customers.clone(),
            domain_configs: domain_configs.clone(),
        };
        Ok(res)
    }

    pub fn for_subject<'s>(
        &'s self,
        sub: &'s <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<DepositsForSubject<'s, Perms, E>, CoreDepositError>
    where
        DepositAccountHolderId:
            for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let holder_id = DepositAccountHolderId::try_from(sub)
            .map_err(|_| CoreDepositError::SubjectIsNotDepositAccountHolder)?;
        Ok(DepositsForSubject::new(
            sub,
            holder_id,
            &self.deposit_accounts,
            &self.deposits,
            &self.withdrawals,
            &self.ledger,
            &self.authz,
        ))
    }

    #[record_error_severity]
    #[instrument(name = "deposit.create_account", skip(self))]
    pub async fn create_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        holder_id: impl Into<DepositAccountHolderId> + Copy + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_CREATE,
            )
            .await?;

        let customer_id = CustomerId::from(holder_id.into());
        let customer = self.customers.find_by_id_without_audit(customer_id).await?;

        let require_verified = self
            .domain_configs
            .get_without_audit::<RequireVerifiedCustomerForAccount>()
            .await?
            .value()
            .unwrap_or(true);
        if require_verified && !customer.kyc_verification.is_verified() {
            return Err(CoreDepositError::CustomerNotVerified);
        }

        let account_id = DepositAccountId::new();

        let mut op = self.deposit_accounts.begin_op().await?;

        let public_id = self
            .public_ids
            .create_in_op(&mut op, DEPOSIT_ACCOUNT_REF_TARGET, account_id)
            .await?;

        let account_ids = DepositAccountLedgerAccountIds::new(account_id);
        let new_account = NewDepositAccount::builder()
            .id(account_id)
            .account_holder_id(holder_id)
            .account_ids(account_ids)
            .public_id(public_id.id)
            .build()
            .expect("Could not build new account");

        let account = self
            .deposit_accounts
            .create_in_op(&mut op, new_account)
            .await?;

        self.ledger
            .create_deposit_accounts(&mut op, &account, customer.customer_type)
            .await?;

        op.commit().await?;

        Ok(account)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_account_by_id", skip(self))]
    pub async fn find_account_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<Option<DepositAccount>, CoreDepositError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(id),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ,
            )
            .await?;

        Ok(self.deposit_accounts.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_account_by_public_id", skip(self))]
    pub async fn find_account_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<DepositAccount>, CoreDepositError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ,
            )
            .await?;

        Ok(self
            .deposit_accounts
            .maybe_find_by_public_id(public_id.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_account_by_id_without_audit", skip(self))]
    pub async fn find_account_by_id_without_audit(
        &self,
        id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let id = id.into();
        Ok(self.deposit_accounts.find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.update_account_status_for_holder", skip(self))]
    pub async fn update_account_status_for_holder(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        holder_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
        status: DepositAccountHolderStatus,
    ) -> Result<(), CoreDepositError> {
        let holder_id = holder_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_UPDATE_STATUS,
            )
            .await?;

        let accounts = self
            .deposit_accounts
            .list_for_account_holder_id_by_id(holder_id, Default::default(), Default::default())
            .await?;
        let mut op = self.deposit_accounts.begin_op().await?;

        for mut account in accounts.entities.into_iter() {
            match account.update_status_via_holder(status) {
                Ok(result) if result.did_execute() => {
                    self.deposit_accounts
                        .update_in_op(&mut op, &mut account)
                        .await?;
                }
                Err(DepositAccountError::CannotUpdateClosedAccount(_)) => {
                    tracing::warn!("Skipping update error if account already closed");
                    continue;
                }
                Err(DepositAccountError::CannotUpdateFrozenAccount(_)) => {
                    tracing::warn!("Skipping update error if account already frozen");
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(_) => continue,
            }
        }
        op.commit().await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "deposit.account_history", skip(self))]
    pub async fn account_history(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<DepositAccountHistoryCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DepositAccountHistoryEntry, DepositAccountHistoryCursor>,
        CoreDepositError,
    > {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ,
            )
            .await?;

        let history = self
            .ledger
            .account_history::<DepositAccountHistoryEntry, DepositAccountHistoryCursor>(
                account_id, query,
            )
            .await?;
        Ok(history)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.record_deposit", skip(self))]
    pub async fn record_deposit(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Deposit, CoreDepositError> {
        let deposit_account_id = deposit_account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposits(),
                CoreDepositAction::DEPOSIT_CREATE,
            )
            .await?;
        self.check_account_active(deposit_account_id).await?;
        let deposit_id = DepositId::new();
        let mut op = self.deposits.begin_op().await?;
        let public_id = self
            .public_ids
            .create_in_op(&mut op, DEPOSIT_REF_TARGET, deposit_id)
            .await?;

        let new_deposit = NewDeposit::builder()
            .id(deposit_id)
            .ledger_transaction_id(deposit_id)
            .deposit_account_id(deposit_account_id)
            .amount(amount)
            .public_id(public_id.id)
            .reference(reference)
            .build()?;
        let deposit = self.deposits.create_in_op(&mut op, new_deposit).await?;
        self.ledger
            .record_deposit(
                &mut op,
                deposit_id,
                amount,
                deposit_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;
        op.commit().await?;
        Ok(deposit)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.initiate_withdrawal", skip(self))]
    pub async fn initiate_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Withdrawal, CoreDepositError> {
        let deposit_account_id = deposit_account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_INITIATE,
            )
            .await?;
        self.check_account_active(deposit_account_id).await?;
        let withdrawal_id = WithdrawalId::new();
        let mut op = self.withdrawals.begin_op().await?;
        let public_id = self
            .public_ids
            .create_in_op(&mut op, WITHDRAWAL_REF_TARGET, withdrawal_id)
            .await?;

        let new_withdrawal = NewWithdrawal::builder()
            .id(withdrawal_id)
            .deposit_account_id(deposit_account_id)
            .amount(amount)
            .approval_process_id(withdrawal_id)
            .public_id(public_id.id)
            .reference(reference)
            .build()?;

        self.governance
            .start_process(
                &mut op,
                withdrawal_id,
                withdrawal_id.to_string(),
                APPROVE_WITHDRAWAL_PROCESS,
            )
            .await?;
        let withdrawal = self
            .withdrawals
            .create_in_op(&mut op, new_withdrawal)
            .await?;

        self.ledger
            .initiate_withdrawal(
                &mut op,
                withdrawal_id,
                amount,
                deposit_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        op.commit().await?;

        Ok(withdrawal)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.revert_deposit", skip(self))]
    pub async fn revert_deposit(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_id: impl Into<DepositId> + std::fmt::Debug,
    ) -> Result<Deposit, CoreDepositError> {
        let id = deposit_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit(id),
                CoreDepositAction::DEPOSIT_REVERT,
            )
            .await?;

        let mut deposit = self.deposits.find_by_id(id).await?;
        self.check_account_active(deposit.deposit_account_id)
            .await?;

        if let es_entity::Idempotent::Executed(deposit_reversal_data) = deposit.revert() {
            let mut op = self.deposits.begin_op().await?;
            self.deposits.update_in_op(&mut op, &mut deposit).await?;
            self.ledger
                .revert_deposit(
                    &mut op,
                    deposit_reversal_data,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
            op.commit().await?;
        }

        Ok(deposit)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.revert_withdrawal", skip(self))]
    pub async fn revert_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, CoreDepositError> {
        let id = withdrawal_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_REVERT,
            )
            .await?;

        let mut withdrawal = self.withdrawals.find_by_id(id).await?;

        self.check_account_active(withdrawal.deposit_account_id)
            .await?;

        if let Ok(es_entity::Idempotent::Executed(withdrawal_reversal_data)) = withdrawal.revert() {
            let mut op = self.withdrawals.begin_op().await?;
            self.withdrawals
                .update_in_op(&mut op, &mut withdrawal)
                .await?;
            self.ledger
                .revert_withdrawal(
                    &mut op,
                    withdrawal_reversal_data,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
            op.commit().await?;
        }

        Ok(withdrawal)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.confirm_withdrawal", skip(self))]
    pub async fn confirm_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, CoreDepositError> {
        let id = withdrawal_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_CONFIRM,
            )
            .await?;
        let mut withdrawal = self.withdrawals.find_by_id(id).await?;
        self.check_account_active(withdrawal.deposit_account_id)
            .await?;
        let mut op = self.withdrawals.begin_op().await?;
        let tx_id = withdrawal.confirm()?;
        self.withdrawals
            .update_in_op(&mut op, &mut withdrawal)
            .await?;

        self.ledger
            .confirm_withdrawal(
                &mut op,
                id,
                tx_id,
                withdrawal.id.to_string(),
                withdrawal.amount,
                withdrawal.deposit_account_id,
                format!("lana:withdraw:{}:confirm", withdrawal.id),
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;

        op.commit().await?;

        Ok(withdrawal)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.cancel_withdrawal", skip(self))]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub async fn cancel_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl es_entity::RetryableInto<WithdrawalId>,
    ) -> Result<Withdrawal, CoreDepositError> {
        let id = withdrawal_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_CANCEL,
            )
            .await?;
        let mut withdrawal = self.withdrawals.find_by_id(id).await?;
        self.check_account_active(withdrawal.deposit_account_id)
            .await?;
        let mut op = self.withdrawals.begin_op().await?;
        let tx_id = withdrawal.cancel()?;
        self.withdrawals
            .update_in_op(&mut op, &mut withdrawal)
            .await?;
        self.ledger
            .cancel_withdrawal(
                &mut op,
                id,
                tx_id,
                withdrawal.amount,
                withdrawal.deposit_account_id,
                LedgerTransactionInitiator::try_from_subject(sub)?,
            )
            .await?;
        op.commit().await?;
        Ok(withdrawal)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.freeze_account", skip(self))]
    pub async fn freeze_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_FREEZE,
            )
            .await?;

        let mut account = self.deposit_accounts.find_by_id(account_id).await?;

        if account.freeze()?.did_execute() {
            let mut op = self.deposit_accounts.begin_op().await?;

            self.deposit_accounts
                .update_in_op(&mut op, &mut account)
                .await?;
            self.ledger
                .freeze_account_in_op(
                    &mut op,
                    &account,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;

            op.commit().await?;
        }

        Ok(account)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.unfreeze_account", skip(self))]
    pub async fn unfreeze_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_UNFREEZE,
            )
            .await?;

        let mut account = self.deposit_accounts.find_by_id(account_id).await?;

        if account.unfreeze()?.did_execute() {
            let mut op = self.deposit_accounts.begin_op().await?;

            self.deposit_accounts
                .update_in_op(&mut op, &mut account)
                .await?;
            self.ledger
                .unfreeze_account_in_op(
                    &mut op,
                    &account,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;

            op.commit().await?;
        }

        Ok(account)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.close_account", skip(self))]
    pub async fn close_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_CLOSE,
            )
            .await?;
        let balance = self.ledger.balance(account_id).await?;
        if !balance.is_zero() {
            return Err(DepositAccountError::BalanceIsNotZero.into());
        }

        let mut account = self.deposit_accounts.find_by_id(account_id).await?;

        if account.close()?.did_execute() {
            let mut op = self.deposit_accounts.begin_op().await?;

            self.deposit_accounts
                .update_in_op(&mut op, &mut account)
                .await?;
            self.ledger.lock_account(&mut op, account_id.into()).await?;

            op.commit().await?;
        }

        Ok(account)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.account_balance", skip(self))]
    pub async fn account_balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<DepositAccountBalance, CoreDepositError> {
        let account_id = account_id.into();
        let _ = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ_BALANCE,
            )
            .await?;

        let balance = self.ledger.balance(account_id).await?;
        Ok(balance)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_deposit_by_id", skip(self))]
    pub async fn find_deposit_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DepositId> + std::fmt::Debug,
    ) -> Result<Option<Deposit>, CoreDepositError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit(id),
                CoreDepositAction::DEPOSIT_READ,
            )
            .await?;

        Ok(self.deposits.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_withdrawal_by_id", skip(self))]
    pub async fn find_withdrawal_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Option<Withdrawal>, CoreDepositError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_READ,
            )
            .await?;

        Ok(self.withdrawals.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_deposit_by_public_id", skip(self))]
    pub async fn find_deposit_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<Deposit>, CoreDepositError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposits(),
                CoreDepositAction::DEPOSIT_READ,
            )
            .await?;

        Ok(self
            .deposits
            .maybe_find_by_public_id(public_id.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_withdrawal_by_public_id", skip(self))]
    pub async fn find_withdrawal_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<Withdrawal>, CoreDepositError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_READ,
            )
            .await?;

        Ok(self
            .withdrawals
            .maybe_find_by_public_id(public_id.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_withdrawal_by_cancelled_tx_id", skip(self))]
    pub async fn find_withdrawal_by_cancelled_tx_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        cancelled_tx_id: impl Into<CalaTransactionId> + std::fmt::Debug,
    ) -> Result<Withdrawal, CoreDepositError> {
        let cancelled_tx_id = cancelled_tx_id.into();
        let withdrawal = self
            .withdrawals
            .find_by_cancelled_tx_id(Some(cancelled_tx_id))
            .await?;
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(withdrawal.id),
                CoreDepositAction::WITHDRAWAL_READ,
            )
            .await?;

        Ok(withdrawal)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_all_withdrawals", skip(self))]
    pub async fn find_all_withdrawals<T: From<Withdrawal>>(
        &self,
        ids: &[WithdrawalId],
    ) -> Result<std::collections::HashMap<WithdrawalId, T>, CoreDepositError> {
        Ok(self.withdrawals.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_all_deposits", skip(self))]
    pub async fn find_all_deposits<T: From<Deposit>>(
        &self,
        ids: &[DepositId],
    ) -> Result<std::collections::HashMap<DepositId, T>, CoreDepositError> {
        Ok(self.deposits.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.find_all_deposit_accounts", skip(self))]
    pub async fn find_all_deposit_accounts<T: From<DepositAccount>>(
        &self,
        ids: &[DepositAccountId],
    ) -> Result<std::collections::HashMap<DepositAccountId, T>, CoreDepositError> {
        Ok(self.deposit_accounts.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.list_withdrawals", skip(self))]
    pub async fn list_withdrawals(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<WithdrawalsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Withdrawal, WithdrawalsByCreatedAtCursor>,
        CoreDepositError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_LIST,
            )
            .await?;
        Ok(self
            .withdrawals
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.list_deposits", skip(self))]
    pub async fn list_deposits(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<DepositsByCreatedAtCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Deposit, DepositsByCreatedAtCursor>, CoreDepositError>
    {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposits(),
                CoreDepositAction::DEPOSIT_LIST,
            )
            .await?;
        Ok(self
            .deposits
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.list_accounts", skip(self))]
    pub async fn list_accounts(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<DepositAccountsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<DepositAccount, DepositAccountsByCreatedAtCursor>,
        CoreDepositError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_LIST,
            )
            .await?;
        Ok(self
            .deposit_accounts
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.list_deposits_for_account", skip(self))]
    pub async fn list_deposits_for_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Deposit>, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposits(),
                CoreDepositAction::DEPOSIT_LIST,
            )
            .await?;
        Ok(self
            .deposits
            .list_for_deposit_account_id_by_created_at(
                account_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[record_error_severity]
    #[instrument(name = "deposit.list_withdrawals_for_account", skip(self))]
    pub async fn list_withdrawals_for_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Withdrawal>, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_LIST,
            )
            .await?;
        Ok(self
            .withdrawals
            .list_for_deposit_account_id_by_created_at(
                account_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit.list_accounts_by_created_at_for_account_holder",
        skip(self)
    )]
    pub async fn list_accounts_by_created_at_for_account_holder(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_holder_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<DepositAccountsByCreatedAtCursor>,
        direction: impl Into<es_entity::ListDirection> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<DepositAccount, DepositAccountsByCreatedAtCursor>,
        CoreDepositError,
    > {
        let account_holder_id = account_holder_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_LIST,
            )
            .await?;

        Ok(self
            .deposit_accounts
            .list_for_account_holder_id_by_created_at(account_holder_id, query, direction.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit.find_account_by_account_holder_without_audit",
        skip(self)
    )]
    pub async fn find_account_by_account_holder_without_audit(
        &self,
        account_holder_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let account_holder_id = account_holder_id.into();
        Ok(self
            .deposit_accounts
            .find_by_account_holder_id(account_holder_id)
            .await?)
    }

    pub async fn get_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, CoreDepositError> {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ,
            )
            .await?;
        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "deposit.set_chart_of_accounts_integration_config",
        skip(self, chart)
    )]
    pub async fn set_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, CoreDepositError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(CoreDepositError::ChartIdMismatch);
        }

        if self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?
            .is_some()
        {
            return Err(CoreDepositError::DepositConfigAlreadyExists);
        }

        let individual_deposit_accounts_parent_account_set_id = chart.account_set_id_from_code(
            &config.chart_of_accounts_individual_deposit_accounts_parent_code,
        )?;
        let government_entity_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_government_entity_deposit_accounts_parent_code,
            )?;
        let private_company_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_private_company_deposit_accounts_parent_code,
            )?;
        let bank_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(&config.chart_of_account_bank_deposit_accounts_parent_code)?;
        let financial_institution_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_financial_institution_deposit_accounts_parent_code,
            )?;
        let non_domiciled_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let frozen_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            )?;
        let frozen_government_entity_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            )?;
        let frozen_private_company_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            )?;
        let frozen_bank_deposit_accounts_parent_account_set_id = chart.account_set_id_from_code(
            &config.chart_of_account_frozen_bank_deposit_accounts_parent_code,
        )?;
        let frozen_financial_institution_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config.chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            )?;
        let frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id = chart
            .account_set_id_from_code(
                &config
                    .chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let omnibus_parent_account_set_id =
            chart.account_set_id_from_code(&config.chart_of_accounts_omnibus_parent_code)?;

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        let charts_integration_meta = ChartOfAccountsIntegrationMeta {
            audit_info,
            config: config.clone(),
            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_individual_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
        };

        self.ledger
            .attach_chart_of_accounts_account_sets(charts_integration_meta)
            .await?;

        Ok(config)
    }

    async fn check_account_active(
        &self,
        deposit_account_id: DepositAccountId,
    ) -> Result<(), CoreDepositError> {
        let account = self.deposit_accounts.find_by_id(deposit_account_id).await?;
        match account.status {
            DepositAccountStatus::Inactive => Err(CoreDepositError::DepositAccountInactive),
            DepositAccountStatus::Frozen => Err(CoreDepositError::DepositAccountFrozen),
            DepositAccountStatus::Closed => Err(CoreDepositError::DepositAccountClosed),
            DepositAccountStatus::Active => Ok(()),
        }
    }
}
