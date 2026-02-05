mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::AccountId as CalaAccountId;
use core_custody::CoreCustodyEvent;
use core_money::{Satoshis, UsdCents};
use governance::GovernanceEvent;
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, LiquidationId,
    PaymentSourceAccountId,
};
pub use entity::NewLiquidationBuilder;
pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
use error::LiquidationError;
pub(crate) use repo::LiquidationRepo;
pub use repo::liquidation_cursor;

pub struct Liquidations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<LiquidationRepo<E>>,
    authz: Arc<Perms>,
}

impl<Perms, E> Clone for Liquidations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms, E> Liquidations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        publisher: &crate::CreditFacilityPublisher<E>,
        clock: es_entity::clock::ClockHandle,
    ) -> Self {
        let repo = Arc::new(LiquidationRepo::new(pool, publisher, clock));
        Self { repo, authz }
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.liquidation.list_for_facility_by_created_at",
        skip(self)
    )]
    pub async fn list_for_facility_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Liquidation>, LiquidationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_for_credit_facility_id_by_created_at(
                credit_facility_id,
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[record_error_severity]
    #[instrument(name = "credit.liquidation.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        liquidation_id: impl Into<LiquidationId> + std::fmt::Debug,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let id = liquidation_id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(id),
                CoreCreditAction::LIQUIDATION_READ,
            )
            .await?;

        self.repo.maybe_find_by_id(id).await
    }

    pub async fn find_all<T: From<Liquidation>>(
        &self,
        ids: &[LiquidationId],
    ) -> Result<std::collections::HashMap<LiquidationId, T>, LiquidationError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "credit.liquidation.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<liquidation_cursor::LiquidationsByIdCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Liquidation, liquidation_cursor::LiquidationsByIdCursor>,
        LiquidationError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_liquidations(),
                CoreCreditAction::LIQUIDATION_LIST,
            )
            .await?;

        self.repo
            .list_by_id(query, es_entity::ListDirection::Descending)
            .await
    }
}

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub amount_received: UsdCents,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
    pub amount_liquidated: Satoshis,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct FacilityProceedsFromLiquidationAccountId(CalaAccountId);

impl FacilityProceedsFromLiquidationAccountId {
    pub fn new() -> Self {
        Self(CalaAccountId::new())
    }

    pub const fn into_inner(self) -> CalaAccountId {
        self.0
    }
}

impl Default for FacilityProceedsFromLiquidationAccountId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&FacilityProceedsFromLiquidationAccountId> for PaymentSourceAccountId {
    fn from(account: &FacilityProceedsFromLiquidationAccountId) -> Self {
        Self::new(account.0)
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for CalaAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        account.0
    }
}
