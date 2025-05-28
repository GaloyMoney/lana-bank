mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, primitives::*, Obligation, Obligations};

pub(super) use entity::*;
use error::DisbursalError;
pub(super) use repo::*;
pub use repo::{DisbursalsSortBy, FindManyDisbursals};

pub use entity::Disbursal;

pub struct Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: DisbursalRepo<E>,
    authz: Perms,
    obligations: Obligations<Perms, E>,
    governance: Governance<Perms, E>,
}

impl<Perms, E> Clone for Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
            obligations: self.obligations.clone(),
        }
    }
}

pub(super) enum ApprovalProcessOutcome {
    Ignored(Disbursal),
    Approved((Disbursal, Obligation)),
    Denied(Disbursal),
}

impl<Perms, E> Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        publisher: &crate::CreditFacilityPublisher<E>,
        obligations: &Obligations<Perms, E>,
        governance: &Governance<Perms, E>,
    ) -> Self {
        let _ = governance
            .init_policy(crate::APPROVE_DISBURSAL_PROCESS)
            .await;

        Self {
            repo: DisbursalRepo::new(pool, publisher),
            authz: authz.clone(),
            obligations: obligations.clone(),
            governance: governance.clone(),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, DisbursalError> {
        Ok(self.repo.begin_op().await?)
    }

    pub(super) async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_disbursal: NewDisbursal,
    ) -> Result<Disbursal, DisbursalError> {
        self.governance
            .start_process(
                db,
                new_disbursal.approval_process_id,
                new_disbursal.approval_process_id.to_string(),
                crate::APPROVE_DISBURSAL_PROCESS,
            )
            .await?;
        let disbursal = self.repo.create_in_op(db, new_disbursal).await?;

        Ok(disbursal)
    }

    pub(super) async fn create_first_disbursal_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_disbursal: NewDisbursal,
        audit_info: &audit::AuditInfo,
    ) -> Result<Disbursal, DisbursalError> {
        let mut disbursal = self.repo.create_in_op(db, new_disbursal).await?;

        let new_obligation = disbursal
            .approval_process_concluded(LedgerTxId::new(), true, audit_info.clone())
            .expect("First instance of idempotent action ignored")
            .expect("First disbursal obligation was already created");

        self.obligations
            .create_with_jobs_in_op(db, new_obligation)
            .await?;

        self.repo.update_in_op(db, &mut disbursal).await?;

        Ok(disbursal)
    }

    #[instrument(name = "core_credit.disbursals.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DisbursalId> + std::fmt::Debug,
    ) -> Result<Option<Disbursal>, DisbursalError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::disbursal(id),
                CoreCreditAction::DISBURSAL_READ,
            )
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(loan) => Ok(Some(loan)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub(super) async fn find_by_concluded_tx_id_without_audit(
        &self,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, DisbursalError> {
        let tx_id = tx_id.into();
        self.repo.find_by_concluded_tx_id(Some(tx_id)).await
    }

    #[instrument(
        name = "core_credit.disbursals.find_by_concluded_tx_id",
        skip(self),
        err
    )]
    pub async fn find_by_concluded_tx_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, DisbursalError> {
        let disbursal = self.find_by_concluded_tx_id_without_audit(tx_id).await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::disbursal(disbursal.id),
                CoreCreditAction::DISBURSAL_READ,
            )
            .await?;

        Ok(disbursal)
    }

    pub(super) async fn conclude_approval_process_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        disbursal_id: DisbursalId,
        approved: bool,
        tx_id: LedgerTxId,
    ) -> Result<ApprovalProcessOutcome, DisbursalError> {
        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::disbursal(disbursal_id),
                CoreCreditAction::DISBURSAL_SETTLE,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let mut disbursal = self.repo.find_by_id(disbursal_id).await?;

        let ret = match disbursal.approval_process_concluded(tx_id, approved, audit_info) {
            es_entity::Idempotent::Ignored => ApprovalProcessOutcome::Ignored(disbursal),
            es_entity::Idempotent::Executed(Some(new_obligation)) => {
                let obligation = self
                    .obligations
                    .create_with_jobs_in_op(db, new_obligation)
                    .await?;
                self.repo.update_in_op(db, &mut disbursal).await?;
                ApprovalProcessOutcome::Approved((disbursal, obligation))
            }
            es_entity::Idempotent::Executed(None) => {
                self.repo.update_in_op(db, &mut disbursal).await?;
                ApprovalProcessOutcome::Denied(disbursal)
            }
        };
        Ok(ret)
    }

    #[instrument(name = "core_credit.disbursals.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<disbursal_cursor::DisbursalsCursor>,
        filter: FindManyDisbursals,
        sort: impl Into<es_entity::Sort<DisbursalsSortBy>> + std::fmt::Debug,
    ) -> Result<
        es_entity::PaginatedQueryRet<Disbursal, disbursal_cursor::DisbursalsCursor>,
        DisbursalError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_disbursals(),
                CoreCreditAction::DISBURSAL_LIST,
            )
            .await?;
        let disbursals = self.repo.find_many(filter, sort.into(), query).await?;

        Ok(disbursals)
    }

    pub(super) async fn list_for_facility_without_audit(
        &self,
        id: CreditFacilityId,
        query: es_entity::PaginatedQueryArgs<disbursal_cursor::DisbursalsCursor>,
        sort: impl Into<es_entity::Sort<DisbursalsSortBy>>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Disbursal, disbursal_cursor::DisbursalsCursor>,
        DisbursalError,
    > {
        self.repo
            .find_many(
                FindManyDisbursals::WithCreditFacilityId(id),
                sort.into(),
                query,
            )
            .await
    }

    #[instrument(name = "core_credit.disbursals.find_all", skip(self), err)]
    pub async fn find_all<T: From<Disbursal>>(
        &self,
        ids: &[DisbursalId],
    ) -> Result<std::collections::HashMap<DisbursalId, T>, DisbursalError> {
        self.repo.find_all(ids).await
    }
}
