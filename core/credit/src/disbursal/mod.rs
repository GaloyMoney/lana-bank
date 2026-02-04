mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;

use crate::{event::CoreCreditEvent, primitives::*};

use core_credit_collection::{CoreCreditCollection, Obligation};

pub(super) use entity::*;
use error::DisbursalError;
pub(super) use repo::*;
pub use repo::{DisbursalsFilter, DisbursalsSortBy};

pub use entity::Disbursal;

#[cfg(feature = "json-schema")]
pub use entity::DisbursalEvent;

pub struct Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    repo: Arc<DisbursalRepo<E>>,
    authz: Arc<Perms>,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    governance: Arc<Governance<Perms, E>>,
}

impl<Perms, E> Clone for Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            governance: self.governance.clone(),
            collections: self.collections.clone(),
        }
    }
}

pub(super) enum ApprovalProcessOutcome {
    AlreadyApplied(Disbursal),
    Approved((Disbursal, Obligation)),
    Denied(Disbursal),
}

impl<Perms, E> Disbursals<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        publisher: &crate::CreditFacilityPublisher<E>,
        collections: Arc<CoreCreditCollection<Perms, E>>,
        governance: Arc<Governance<Perms, E>>,
        clock: es_entity::clock::ClockHandle,
    ) -> Result<Self, DisbursalError> {
        governance
            .init_policy(crate::APPROVE_DISBURSAL_PROCESS)
            .await?;

        Ok(Self {
            repo: Arc::new(DisbursalRepo::new(pool, publisher, clock)),
            authz,
            collections,
            governance,
        })
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
            .start_process_in_op(
                db,
                new_disbursal.approval_process_id,
                new_disbursal.approval_process_id.to_string(),
                crate::APPROVE_DISBURSAL_PROCESS,
            )
            .await?;
        let disbursal = self.repo.create_in_op(db, new_disbursal).await?;

        Ok(disbursal)
    }

    #[record_error_severity]
    #[instrument(
        name = "disbursals.create_pre_approved_disbursal_in_op",
        skip(self, db, new_disbursal)
    )]
    pub(super) async fn create_pre_approved_disbursal_in_op(
        &self,
        db: &mut es_entity::DbOpWithTime<'_>,
        new_disbursal: NewDisbursal,
    ) -> Result<Disbursal, DisbursalError> {
        let mut disbursal = self.repo.create_in_op(db, new_disbursal).await?;

        let new_obligation = disbursal
            .approval_process_concluded_for_initial_disbursal(db.now().date_naive())
            .expect("First instance of idempotent action ignored")
            .expect("First disbursal obligation was already created");

        self.collections
            .obligations()
            .create_with_jobs_in_op(db, new_obligation)
            .await?;

        self.repo.update_in_op(db, &mut disbursal).await?;

        Ok(disbursal)
    }

    #[record_error_severity]
    #[instrument(name = "core_credit.disbursals.find_by_id", skip(self))]
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

        self.repo.maybe_find_by_id(id).await
    }

    pub(super) async fn find_by_concluded_tx_id_without_audit(
        &self,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, DisbursalError> {
        let tx_id = tx_id.into();
        self.repo.find_by_concluded_tx_id(Some(tx_id)).await
    }

    #[record_error_severity]
    #[instrument(name = "core_credit.disbursals.find_by_concluded_tx_id", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "core_credit.disbursals.find_by_public_id", skip(self))]
    pub async fn find_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<Disbursal>, DisbursalError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_disbursals(),
                CoreCreditAction::DISBURSAL_READ,
            )
            .await?;

        self.repo.maybe_find_by_public_id(public_id.into()).await
    }

    pub(super) async fn conclude_approval_process_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        disbursal_id: DisbursalId,
        approved: bool,
    ) -> Result<ApprovalProcessOutcome, DisbursalError> {
        self.authz
            .audit()
            .record_system_entry_in_op(
                op,
                CoreCreditObject::disbursal(disbursal_id),
                CoreCreditAction::DISBURSAL_SETTLE,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let mut disbursal = self.repo.find_by_id(disbursal_id).await?;

        let ret = match disbursal.approval_process_concluded(approved, op.now().date_naive()) {
            es_entity::Idempotent::AlreadyApplied => {
                ApprovalProcessOutcome::AlreadyApplied(disbursal)
            }
            es_entity::Idempotent::Executed(Some(new_obligation)) => {
                let obligation = self
                    .collections
                    .obligations()
                    .create_with_jobs_in_op(op, new_obligation)
                    .await?;
                self.repo.update_in_op(op, &mut disbursal).await?;
                ApprovalProcessOutcome::Approved((disbursal, obligation))
            }
            es_entity::Idempotent::Executed(None) => {
                self.repo.update_in_op(op, &mut disbursal).await?;
                ApprovalProcessOutcome::Denied(disbursal)
            }
        };
        Ok(ret)
    }

    #[record_error_severity]
    #[instrument(name = "core_credit.disbursals.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<disbursal_cursor::DisbursalsCursor>,
        filter: DisbursalsFilter,
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
        let disbursals = self
            .repo
            .list_for_filter(filter, sort.into(), query)
            .await?;

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
            .list_for_filter(
                DisbursalsFilter::WithCreditFacilityId(id),
                sort.into(),
                query,
            )
            .await
    }

    #[record_error_severity]
    #[instrument(name = "core_credit.disbursals.find_all", skip(self))]
    pub async fn find_all<T: From<Disbursal>>(
        &self,
        ids: &[DisbursalId],
    ) -> Result<std::collections::HashMap<DisbursalId, T>, DisbursalError> {
        self.repo.find_all(ids).await
    }
}
