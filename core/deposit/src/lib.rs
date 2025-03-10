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
mod withdrawal;

use deposit_account_cursor::DepositAccountsByCreatedAtCursor;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chart_of_accounts::new::Chart;
use core_customer::{CoreCustomerEvent, Customers};
use governance::{Governance, GovernanceEvent};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

pub use account::DepositAccount;
use account::*;
pub use chart_of_accounts_integration::ChartOfAccountsIntegrationConfig;
use deposit::*;
pub use deposit::{Deposit, DepositsByCreatedAtCursor};
pub use deposit_account_balance::DepositAccountBalance;
use error::*;
pub use event::*;
pub use for_subject::DepositsForSubject;
pub use history::{DepositAccountHistoryCursor, DepositAccountHistoryEntry};
use ledger::*;
pub use primitives::*;
pub use processes::approval::APPROVE_WITHDRAWAL_PROCESS;
use processes::approval::{
    ApproveWithdrawal, WithdrawApprovalJobConfig, WithdrawApprovalJobInitializer,
};
use withdrawal::*;
pub use withdrawal::{Withdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor};

pub struct CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    accounts: DepositAccountRepo,
    deposits: DepositRepo,
    withdrawals: WithdrawalRepo,
    approve_withdrawal: ApproveWithdrawal<Perms, E>,
    ledger: DepositLedger,
    cala: CalaLedger,
    authz: Perms,
    governance: Governance<Perms, E>,
    customers: Customers<Perms, E>,
    outbox: Outbox<E>,
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
            accounts: self.accounts.clone(),
            deposits: self.deposits.clone(),
            withdrawals: self.withdrawals.clone(),
            ledger: self.ledger.clone(),
            cala: self.cala.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
            customers: self.customers.clone(),
            approve_withdrawal: self.approve_withdrawal.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustomerEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
        governance: &Governance<Perms, E>,
        customers: &Customers<Perms, E>,
        jobs: &Jobs,
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
    ) -> Result<Self, CoreDepositError> {
        let accounts = DepositAccountRepo::new(pool);
        let deposits = DepositRepo::new(pool);
        let withdrawals = WithdrawalRepo::new(pool);
        let ledger = DepositLedger::init(cala, journal_id).await?;

        let approve_withdrawal = ApproveWithdrawal::new(&withdrawals, authz.audit(), governance);

        jobs.add_initializer_and_spawn_unique(
            WithdrawApprovalJobInitializer::new(outbox, &approve_withdrawal),
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
            accounts,
            deposits,
            withdrawals,
            authz: authz.clone(),
            outbox: outbox.clone(),
            governance: governance.clone(),
            customers: customers.clone(),
            cala: cala.clone(),
            approve_withdrawal,
            ledger,
        };
        Ok(res)
    }

    pub fn for_subject<'s>(
        &'s self,
        sub: &'s <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<DepositsForSubject<'s, Perms>, CoreDepositError>
    where
        DepositAccountHolderId:
            for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let holder_id = DepositAccountHolderId::try_from(sub)
            .map_err(|_| CoreDepositError::SubjectIsNotDepositAccountHolder)?;
        Ok(DepositsForSubject::new(
            sub,
            holder_id,
            &self.accounts,
            &self.deposits,
            &self.withdrawals,
            &self.ledger,
            &self.authz,
        ))
    }

    #[instrument(name = "deposit.create_account", skip(self), err)]
    pub async fn create_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        holder_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
        reference: &str,
        name: &str,
        description: &str,
    ) -> Result<DepositAccount, CoreDepositError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_CREATE,
            )
            .await?;

        let account_id = DepositAccountId::new();
        let new_account = NewDepositAccount::builder()
            .id(account_id)
            .account_holder_id(holder_id)
            .reference(reference.to_string())
            .name(name.to_string())
            .description(description.to_string())
            .audit_info(audit_info.clone())
            .build()
            .expect("Could not build new account");

        let mut op = self.accounts.begin_op().await?;
        let account = self.accounts.create_in_op(&mut op, new_account).await?;

        self.ledger
            .create_deposit_account(
                op,
                account_id,
                account.reference.to_string(),
                account.name.to_string(),
                account.description.to_string(),
            )
            .await?;

        Ok(account)
    }

    #[instrument(name = "deposit.for_subject.account_history", skip(self), err)]
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

    #[instrument(name = "deposit.record_deposit", skip(self), err)]
    pub async fn record_deposit(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Deposit, CoreDepositError> {
        let deposit_account_id = deposit_account_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposits(),
                CoreDepositAction::DEPOSIT_CREATE,
            )
            .await?;
        let deposit_id = DepositId::new();
        let new_deposit = NewDeposit::builder()
            .id(deposit_id)
            .ledger_transaction_id(deposit_id)
            .deposit_account_id(deposit_account_id)
            .amount(amount)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new account");

        let mut op = self.deposits.begin_op().await?;
        let deposit = self.deposits.create_in_op(&mut op, new_deposit).await?;
        self.ledger
            .record_deposit(op, deposit_id, amount, deposit_account_id)
            .await?;
        Ok(deposit)
    }

    #[instrument(name = "deposit.initiate_withdrawal", skip(self), err)]
    pub async fn initiate_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_account_id: impl Into<DepositAccountId> + std::fmt::Debug,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Withdrawal, CoreDepositError> {
        let deposit_account_id = deposit_account_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_INITIATE,
            )
            .await?;
        let withdrawal_id = WithdrawalId::new();
        let new_withdrawal = NewWithdrawal::builder()
            .id(withdrawal_id)
            .deposit_account_id(deposit_account_id)
            .amount(amount)
            .approval_process_id(withdrawal_id)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new withdrawal");

        let mut op = self.withdrawals.begin_op().await?;
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
            .initiate_withdrawal(op, withdrawal_id, amount, deposit_account_id)
            .await?;
        Ok(withdrawal)
    }

    #[instrument(name = "deposit.confirm_withdrawal", skip(self), err)]
    pub async fn confirm_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, CoreDepositError> {
        let id = withdrawal_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_CONFIRM,
            )
            .await?;
        let mut withdrawal = self.withdrawals.find_by_id(id).await?;
        let mut op = self.withdrawals.begin_op().await?;
        let tx_id = withdrawal.confirm(audit_info)?;
        self.withdrawals
            .update_in_op(&mut op, &mut withdrawal)
            .await?;

        self.ledger
            .confirm_withdrawal(
                op,
                tx_id,
                withdrawal.id.to_string(),
                withdrawal.amount,
                withdrawal.deposit_account_id,
                format!("lana:withdraw:{}:confirm", withdrawal.id),
            )
            .await?;

        Ok(withdrawal)
    }

    #[instrument(name = "deposit.cancel_withdrawal", skip(self), err)]
    pub async fn cancel_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, CoreDepositError> {
        let id = withdrawal_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_CANCEL,
            )
            .await?;
        let mut withdrawal = self.withdrawals.find_by_id(id).await?;
        let mut op = self.withdrawals.begin_op().await?;
        let tx_id = withdrawal.cancel(audit_info)?;
        self.withdrawals
            .update_in_op(&mut op, &mut withdrawal)
            .await?;
        self.ledger
            .cancel_withdrawal(op, tx_id, withdrawal.amount, withdrawal.deposit_account_id)
            .await?;
        Ok(withdrawal)
    }

    #[instrument(name = "deposit.account_balance", skip(self), err)]
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

    #[instrument(name = "deposit.find_deposit_by_id", skip(self), err)]
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

        match self.deposits.find_by_id(id).await {
            Ok(deposit) => Ok(Some(deposit)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "deposit.find_withdrawal_by_id", skip(self), err)]
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

        match self.withdrawals.find_by_id(id).await {
            Ok(withdrawal) => Ok(Some(withdrawal)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn find_withdrawal_by_cancelled_tx_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        cancelled_tx_id: impl Into<LedgerTransactionId> + std::fmt::Debug,
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

    #[instrument(name = "deposit.find_all_withdrawals", skip(self), err)]
    pub async fn find_all_withdrawals<T: From<Withdrawal>>(
        &self,
        ids: &[WithdrawalId],
    ) -> Result<std::collections::HashMap<WithdrawalId, T>, CoreDepositError> {
        Ok(self.withdrawals.find_all(ids).await?)
    }

    #[instrument(name = "deposit.find_all_deposits", skip(self), err)]
    pub async fn find_all_deposits<T: From<Deposit>>(
        &self,
        ids: &[DepositId],
    ) -> Result<std::collections::HashMap<DepositId, T>, CoreDepositError> {
        Ok(self.deposits.find_all(ids).await?)
    }

    #[instrument(name = "deposit.find_all_deposit_accounts", skip(self), err)]
    pub async fn find_all_deposit_accounts<T: From<DepositAccount>>(
        &self,
        ids: &[DepositAccountId],
    ) -> Result<std::collections::HashMap<DepositAccountId, T>, CoreDepositError> {
        Ok(self.accounts.find_all(ids).await?)
    }

    #[instrument(name = "deposit.list_withdrawals", skip(self), err)]
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

    #[instrument(name = "deposit.list_deposits", skip(self), err)]
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

    #[instrument(name = "deposit.ensure_up_to_date_status", skip(self, withdraw), err)]
    pub async fn ensure_up_to_date_status(
        &self,
        withdraw: &Withdrawal,
    ) -> Result<Option<Withdrawal>, CoreDepositError> {
        Ok(self.approve_withdrawal.execute_from_svc(withdraw).await?)
    }

    #[instrument(name = "deposit.list_deposits_for_account", skip(self), err)]
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

    #[instrument(name = "deposit.list_withdrawals_for_account", skip(self), err)]
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

    #[instrument(
        name = "deposit.list_accounts_by_created_at_for_account_holder",
        skip(self),
        err
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
            .accounts
            .list_for_account_holder_id_by_created_at(account_holder_id, query, direction.into())
            .await?)
    }

    pub async fn update_chart_of_accounts_integration_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, CoreDepositError> {
        let deposit_accounts_parent_account_set_id = if let Some((_, id)) =
            chart.account_spec(&config.chart_of_accounts_deposit_accounts_parent_code)
        {
            id
        } else {
            return Err(CoreDepositError::CodeNotFoundInChart(
                config.chart_of_accounts_deposit_accounts_parent_code,
            ));
        };
        let omnibus_parent_account_set_id = if let Some((_, id)) =
            chart.account_spec(&config.chart_of_accounts_omnibus_parent_code)
        {
            id
        } else {
            return Err(CoreDepositError::CodeNotFoundInChart(
                config.chart_of_accounts_omnibus_parent_code,
            ));
        };
        if chart.id != config.chart_of_accounts_id {
            return Err(CoreDepositError::ChartIdMismatch);
        }

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::chart_of_accounts_integration(),
                CoreDepositAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        self.ledger
            .attach_chart_of_accounts_account_sets(
                audit_info,
                &config,
                *deposit_accounts_parent_account_set_id,
                *omnibus_parent_account_set_id,
            )
            .await?;

        Ok(config)
    }
}
