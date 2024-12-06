#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod account;
mod deposit;
pub mod error;
mod event;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};

use account::*;
use deposit::*;
use error::*;
pub use event::*;
pub use primitives::*;

pub struct CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent>,
{
    accounts: DepositAccountRepo,
    deposits: DepositRepo,
    authz: Perms,
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
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
        }
    }
}

impl<Perms, E> CoreDeposit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDepositAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDepositObject>,
    E: OutboxEventMarker<CoreDepositEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
    ) -> Result<Self, CoreDepositError> {
        let accounts = DepositAccountRepo::new(pool);
        let deposits = DepositRepo::new(pool);
        let res = Self {
            accounts,
            deposits,
            authz: authz.clone(),
            outbox: outbox.clone(),
        };
        Ok(res)
    }

    #[instrument(name = "deposit.create_account", skip(self))]
    pub async fn create_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        holder_id: impl Into<AccountHolderId> + std::fmt::Debug,
    ) -> Result<DepositAccount, CoreDepositError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_deposit_accounts(),
                CoreDepositAction::DEPOSIT_ACCOUNT_CREATE,
            )
            .await?;

        let new_account = NewDepositAccount::builder()
            .id(DepositAccountId::new())
            .account_holder_id(holder_id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let account = self.accounts.create(new_account).await?;
        Ok(account)
    }

    #[instrument(name = "deposit.record_deposit", skip(self))]
    pub async fn record_deposit(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        deposit_account_id: DepositAccountId,
        // amount: UsdCents,
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

        let new_deposit = NewDeposit::builder()
            .id(DepositId::new())
            .deposit_account_id(deposit_account_id)
            .reference(reference)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let deposit = self.deposits.create(new_deposit).await?;
        Ok(deposit)
    }

    pub async fn balance(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: DepositAccountId,
    ) -> Result<(), CoreDepositError> {
        let _ = self
            .authz
            .enforce_permission(
                sub,
                CoreDepositObject::deposit_account(account_id),
                CoreDepositAction::DEPOSIT_ACCOUNT_READ_BALANCE,
            )
            .await?;

        // let balance = self.accounts.balance(account_id).await?;
        // Ok(balance)
        Ok(())
    }
}
