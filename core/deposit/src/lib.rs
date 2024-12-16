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
use governance::{Governance, GovernanceEvent};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker};

use account::*;
use deposit::*;
use deposit_account_balance::*;
use error::*;
pub use event::*;
use ledger::*;
pub use primitives::*;
use processes::approval::{
    ApproveWithdrawal, WithdrawApprovalJobConfig, WithdrawApprovalJobInitializer,
    APPROVE_WITHDRAWAL_PROCESS,
};
use withdrawal::*;

pub struct CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>,
{
    accounts: DepositAccountRepo,
    deposits: DepositRepo,
    withdrawals: WithdrawalRepo,
    ledger: DepositLedger,
    authz: Perms,
    governance: Governance<Perms, E>,
    outbox: Outbox<E>,
}

impl<Perms, E> Clone for CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>,
{
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            deposits: self.deposits.clone(),
            withdrawals: self.withdrawals.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
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
        cala: &CalaLedger,
        journal_id: LedgerJournalId,
        omnibus_account_code: String,
    ) -> Result<Self, CoreDepositError> {
        let accounts = DepositAccountRepo::new(pool);
        let deposits = DepositRepo::new(pool);
        let withdrawals = WithdrawalRepo::new(pool);
        let ledger = DepositLedger::init(cala, journal_id, omnibus_account_code).await?;

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
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Deposit, CoreDepositError> {
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
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
        reference: Option<String>,
    ) -> Result<Withdrawal, CoreDepositError> {
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
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_CONFIRM,
            )
            .await?;
        let id = withdrawal_id.into();
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
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_CANCEL,
            )
            .await?;
        let id = withdrawal_id.into();
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

    pub async fn balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: DepositAccountId,
    ) -> Result<DepositAccountBalance, CoreDepositError> {
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
}
