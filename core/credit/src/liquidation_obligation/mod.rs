mod entity;
pub mod error;
mod primitives;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use job::{JobId, Jobs};
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    liquidation_obligation_defaulted,
    primitives::{CoreCreditAction, CoreCreditObject, LiquidationObligationId},
    publisher::CreditFacilityPublisher,
};

pub(crate) use entity::*;
pub(crate) use error::LiquidationObligationError;
pub(crate) use primitives::LiquidationObligationDefaultedReallocationData;
pub(crate) use repo::LiquidationObligationRepo;

pub struct LiquidationObligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Perms,
    liquidation_obligation_repo: LiquidationObligationRepo<E>,
    jobs: Jobs,
}

impl<Perms, E> Clone for LiquidationObligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            liquidation_obligation_repo: self.liquidation_obligation_repo.clone(),
            jobs: self.jobs.clone(),
        }
    }
}

impl<Perms, E> LiquidationObligations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub(crate) fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        _cala: &CalaLedger,
        jobs: &Jobs,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let liquidation_obligation_repo = LiquidationObligationRepo::new(pool, publisher);
        Self {
            authz: authz.clone(),
            liquidation_obligation_repo,
            jobs: jobs.clone(),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, LiquidationObligationError> {
        Ok(self.liquidation_obligation_repo.begin_op().await?)
    }

    pub async fn create_with_jobs_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_liquidation_obligation: NewLiquidationObligation,
    ) -> Result<LiquidationObligation, LiquidationObligationError> {
        let liquidation_obligation = self
            .liquidation_obligation_repo
            .create_in_op(db, new_liquidation_obligation)
            .await?;
        if let Some(defaulted_at) = liquidation_obligation.defaulted_at() {
            self.jobs
                .create_and_spawn_at_in_op(
                    db,
                    JobId::new(),
                    liquidation_obligation_defaulted::CreditFacilityJobConfig::<Perms, E> {
                        liquidation_obligation_id: liquidation_obligation.id,
                        effective: defaulted_at.date_naive(),
                        _phantom: std::marker::PhantomData,
                    },
                    defaulted_at,
                )
                .await?;
        }

        Ok(liquidation_obligation)
    }

    pub async fn record_defaulted_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: LiquidationObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<Option<LiquidationObligationDefaultedReallocationData>, LiquidationObligationError>
    {
        let mut liquidation_obligation = self.liquidation_obligation_repo.find_by_id(id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::liquidation_obligation(id),
                CoreCreditAction::LIQUIDATION_OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(defaulted) =
            liquidation_obligation.record_defaulted(effective, audit_info)?
        {
            self.liquidation_obligation_repo
                .update_in_op(db, &mut liquidation_obligation)
                .await?;
            Some(defaulted)
        } else {
            None
        };

        Ok(data)
    }

    pub async fn find_by_id(
        &self,
        id: LiquidationObligationId,
    ) -> Result<LiquidationObligation, LiquidationObligationError> {
        self.liquidation_obligation_repo.find_by_id(id).await
    }
}
