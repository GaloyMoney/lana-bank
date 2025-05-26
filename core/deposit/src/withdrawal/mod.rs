mod entity;
pub mod error;
pub mod repo;

use tracing::instrument;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use governance::GovernanceEvent;
use outbox::OutboxEventMarker;

use super::primitives::*;

pub(super) use entity::*;
pub use entity::{Withdrawal, WithdrawalStatus};
use error::*;
pub use repo::withdrawal_cursor::WithdrawalsByCreatedAtCursor;
pub(super) use repo::*;

use crate::CoreDepositEvent;

pub struct Withdrawals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: WithdrawalRepo<E>,
    authz: Perms,
}

impl<Perms, E> Clone for Withdrawals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms, E> Withdrawals<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDepositAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreDepositObject>,
    E: OutboxEventMarker<CoreDepositEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, publisher: &crate::DepositPublisher<E>) -> Self {
        Self {
            repo: WithdrawalRepo::new(pool, publisher),
            authz: authz.clone(),
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, WithdrawalError> {
        let res = self.repo.begin_op().await?;
        Ok(res)
    }

    #[instrument(name = "core_deposit.withdrawal.create", skip(self, op), err)]
    pub(super) async fn create(
        &self,
        op: &mut es_entity::DbOp<'_>,
        new_withdrawal: NewWithdrawal,
    ) -> Result<Withdrawal, WithdrawalError> {
        self.repo.create_in_op(op, new_withdrawal).await
    }

    #[instrument(
        name = "core_deposit.withdrawal.confirm",
        skip(self, withdrawal, op),
        err
    )]
    pub(super) async fn confirm(
        &self,
        op: &mut es_entity::DbOp<'_>,
        withdrawal: &mut Withdrawal,
        audit_info: AuditInfo,
    ) -> Result<CalaTransactionId, WithdrawalError> {
        let tx_id = withdrawal.confirm(audit_info)?;
        self.repo.update_in_op(op, withdrawal).await?;

        Ok(tx_id)
    }

    #[instrument(
        name = "core_deposit.withdrawal.cancel",
        skip(self, withdrawal, op),
        err
    )]
    pub(super) async fn cancel(
        &self,
        op: &mut es_entity::DbOp<'_>,
        withdrawal: &mut Withdrawal,
        audit_info: AuditInfo,
    ) -> Result<CalaTransactionId, WithdrawalError> {
        let tx_id = withdrawal.cancel(audit_info)?;
        self.repo.update_in_op(op, withdrawal).await?;

        Ok(tx_id)
    }

    #[instrument(
        name = "core_deposit.withdrawal.conclude_approval_process",
        skip(self),
        err
    )]
    pub(super) async fn conclude_approval_process(
        &self,
        id: impl es_entity::RetryableInto<WithdrawalId>,
        approved: bool,
    ) -> Result<Withdrawal, WithdrawalError> {
        let id = id.into();
        let mut withdraw = self.repo.find_by_id(id).await?;
        if withdraw.is_approved_or_denied().is_some() {
            return Ok(withdraw);
        }

        let mut db = self.repo.begin_op().await?;
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::Withdrawal(WithdrawalAction::ConcludeApprovalProcess),
            )
            .await?;
        if withdraw
            .approval_process_concluded(approved, audit_info)
            .did_execute()
        {
            self.repo.update_in_op(&mut db, &mut withdraw).await?;
            db.commit().await?;
        }

        Ok(withdraw)
    }

    pub(super) async fn find_by_id_without_audit(
        &self,
        id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Withdrawal, WithdrawalError> {
        let id = id.into();
        self.repo.find_by_id(id).await
    }

    #[instrument(name = "core_deposit.withdrawal.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<WithdrawalId> + std::fmt::Debug,
    ) -> Result<Option<Withdrawal>, WithdrawalError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::withdrawal(id),
                CoreDepositAction::WITHDRAWAL_READ,
            )
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(withdrawal) => Ok(Some(withdrawal)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(super) async fn find_by_cancelled_tx_id_without_audit(
        &self,
        cancelled_tx_id: impl Into<CalaTransactionId> + std::fmt::Debug,
    ) -> Result<Withdrawal, WithdrawalError> {
        let cancelled_tx_id = cancelled_tx_id.into();
        self.repo
            .find_by_cancelled_tx_id(Some(cancelled_tx_id))
            .await
    }

    #[instrument(
        name = "core_deposit.withdrawal.find_by_cancelled_tx_id",
        skip(self),
        err
    )]
    pub async fn find_by_cancelled_tx_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        cancelled_tx_id: impl Into<CalaTransactionId> + std::fmt::Debug,
    ) -> Result<Withdrawal, WithdrawalError> {
        let cancelled_tx_id = cancelled_tx_id.into();
        let withdrawal = self
            .repo
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

    #[instrument(
        name = "core_deposit.withdrawal.list_for_account_internal",
        skip(self),
        err
    )]
    pub(super) async fn list_for_account_without_audit(
        &self,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Withdrawal>, WithdrawalError> {
        let account_id = account_id.into();
        Ok(self
            .repo
            .list_for_deposit_account_id_by_created_at(
                account_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[instrument(name = "core_deposit.withdrawal.list_for_account", skip(self), err)]
    pub async fn list_for_account(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        account_id: impl Into<DepositAccountId> + std::fmt::Debug,
    ) -> Result<Vec<Withdrawal>, WithdrawalError> {
        let account_id = account_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_LIST,
            )
            .await?;
        self.list_for_account_without_audit(account_id).await
    }

    #[instrument(name = "core_deposit.withdrawal.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<WithdrawalsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Withdrawal, WithdrawalsByCreatedAtCursor>,
        WithdrawalError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreDepositObject::all_withdrawals(),
                CoreDepositAction::WITHDRAWAL_LIST,
            )
            .await?;

        self.repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await
    }

    #[instrument(name = "core_deposit.withdrawal.find_all", skip(self), err)]
    pub async fn find_all<T: From<Withdrawal>>(
        &self,
        ids: &[WithdrawalId],
    ) -> Result<std::collections::HashMap<WithdrawalId, T>, WithdrawalError> {
        self.repo.find_all(ids).await
    }
}
