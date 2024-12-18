#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod account;
mod deposit;
mod deposit_account_balance;
pub mod error;
mod event;
mod ledger;
mod primitives;
mod processes;
mod withdrawal;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use chart_of_accounts::CoreChartOfAccounts;
use governance::{Governance, GovernanceEvent};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

pub use account::DepositAccount;
use account::*;
use deposit::*;
pub use deposit::{Deposit, DepositsByCreatedAtCursor};
pub use deposit_account_balance::DepositAccountBalance;
use error::*;
pub use event::*;
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
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    accounts: DepositAccountRepo,
    deposits: DepositRepo,
    withdrawals: WithdrawalRepo,
    approve_withdrawal: ApproveWithdrawal<Perms, E>,
    ledger: DepositLedger,
    authz: Perms,
    governance: Governance<Perms, E>,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            deposits: self.deposits.clone(),
            withdrawals: self.withdrawals.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
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
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
        governance: &Governance<Perms, E>,
        jobs: &Jobs,
        _chart_of_accounts: &CoreChartOfAccounts<Perms>,
        // chart_id: ChartId
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
        omnibus_account_code: String,
    ) -> Result<Self, CoreDepositError> {
        let accounts = DepositAccountRepo::new(pool);
        let deposits = DepositRepo::new(pool);
        let withdrawals = WithdrawalRepo::new(pool);
        let ledger = DepositLedger::init(cala, journal_id, omnibus_account_code).await?;

        let approve_withdrawal = ApproveWithdrawal::new(&withdrawals, authz.audit(), governance);
        // chart_of_accounts.create_control_account()
        // chart_of_accounts.create_sub_account()

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
            approve_withdrawal,
            ledger,
        };
        Ok(res)
    }

    #[instrument(name = "deposit.create_account", skip(self))]
    pub async fn create_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        holder_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
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
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let mut op = self.accounts.begin_op().await?;
        let account = self.accounts.create_in_op(&mut op, new_account).await?;
        self.ledger
            .create_account_for_deposit_account(op, account_id, account_id.to_string())
            .await?;
        Ok(account)
    }

    #[instrument(name = "deposit.record_deposit", skip(self))]
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
            .deposit_account_id(deposit_account_id)
            .amount(amount)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let mut op = self.deposits.begin_op().await?;
        let deposit = self.deposits.create_in_op(&mut op, new_deposit).await?;
        self.ledger
            .record_deposit(op, deposit_id, amount, deposit_account_id)
            .await?;
        Ok(deposit)
    }

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

        // TODO: add approval process and check for balance
        self.ledger
            .initiate_withdrawal(op, withdrawal_id, amount, deposit_account_id)
            .await?;
        Ok(withdrawal)
    }

    pub async fn confirm_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId>,
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

    pub async fn cancel_withdrawal(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        withdrawal_id: impl Into<WithdrawalId>,
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

    pub async fn find_all_withdrawals<T: From<Withdrawal>>(
        &self,
        ids: &[WithdrawalId],
    ) -> Result<std::collections::HashMap<WithdrawalId, T>, CoreDepositError> {
        Ok(self.withdrawals.find_all(ids).await?)
    }

    pub async fn find_all_deposits<T: From<Deposit>>(
        &self,
        ids: &[DepositId],
    ) -> Result<std::collections::HashMap<DepositId, T>, CoreDepositError> {
        Ok(self.deposits.find_all(ids).await?)
    }

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

    pub async fn ensure_up_to_date_status(
        &self,
        withdraw: &Withdrawal,
    ) -> Result<Option<Withdrawal>, CoreDepositError> {
        Ok(self.approve_withdrawal.execute_from_svc(withdraw).await?)
    }

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

    pub async fn find_account_for_account_holder(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountHolderId> + std::fmt::Debug,
    ) -> Result<Option<DepositAccount>, CoreDepositError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ,
            )
            .await?;
        match self.accounts.find_by_account_holder_id(account_id).await {
            Ok(account) => Ok(Some(account)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
