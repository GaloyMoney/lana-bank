mod entity;
pub mod error;
mod jobs;
mod ledger;
mod repo;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, JournalId, TransactionId as CalaTransactionId,
};
use core_accounting::LedgerTransactionInitiator;
use core_custody::CoreCustodyEvent;
use core_money::{Satoshis, UsdCents};
use es_entity::Idempotent;
use governance::GovernanceEvent;
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, LedgerOmnibusAccountIds,
    LiquidationId, PaymentId, PaymentSourceAccountId,
};
use entity::NewLiquidationBuilder;
pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
use error::LiquidationError;
use ledger::LiquidationLedger;
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
    ledger: LiquidationLedger,
    proceeds_omnibus_account_ids: LedgerOmnibusAccountIds,
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
            ledger: self.ledger.clone(),
            proceeds_omnibus_account_ids: self.proceeds_omnibus_account_ids.clone(),
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
    pub async fn init(
        pool: &sqlx::PgPool,
        journal_id: JournalId,
        cala: &CalaLedger,
        proceeds_omnibus_account_ids: &LedgerOmnibusAccountIds,
        authz: Arc<Perms>,
        publisher: &crate::CreditFacilityPublisher<E>,
        jobs: &mut job::Jobs,
        outbox: &obix::Outbox<E>,
    ) -> Result<Self, LiquidationError> {
        let repo_arc = Arc::new(LiquidationRepo::new(pool, publisher));

        let _credit_facility_liquidations_job_spawner = jobs.add_initializer(
            jobs::credit_facility_liquidations::CreditFacilityLiquidationsInit::new(
                outbox,
                repo_arc.clone(),
                proceeds_omnibus_account_ids,
            ),
        );

        Ok(Self {
            repo: repo_arc,
            authz,
            ledger: LiquidationLedger::init(cala, journal_id).await?,
            proceeds_omnibus_account_ids: proceeds_omnibus_account_ids.clone(),
        })
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, LiquidationError> {
        Ok(self.repo.begin_op().await?)
    }

    #[instrument(
        name = "credit.liquidation.record_collateral_sent",
        skip(self, sub),
        err
    )]
    pub async fn record_collateral_sent(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        liquidation_id: LiquidationId,
        amount: Satoshis,
    ) -> Result<Liquidation, LiquidationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(liquidation_id),
                CoreCreditAction::LIQUIDATION_RECORD_COLLATERAL_SENT,
            )
            .await?;
        let mut db = self.repo.begin_op().await?;

        let mut liquidation = self.repo.find_by_id_in_op(&mut db, liquidation_id).await?;

        let tx_id = CalaTransactionId::new();

        if liquidation
            .record_collateral_sent_out(amount, tx_id)?
            .did_execute()
        {
            self.repo.update_in_op(&mut db, &mut liquidation).await?;
            self.ledger
                .record_collateral_sent_in_op(
                    &mut db,
                    tx_id,
                    amount,
                    liquidation.collateral_account_id,
                    liquidation.collateral_in_liquidation_account_id,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
        }

        db.commit().await?;

        Ok(liquidation)
    }

    #[instrument(
        name = "credit.liquidation.record_proceeds_from_liquidation",
        skip(self, sub),
        err
    )]
    pub async fn record_proceeds_from_liquidation(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        liquidation_id: LiquidationId,
        amount_received: UsdCents,
    ) -> Result<Liquidation, LiquidationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::liquidation(liquidation_id),
                CoreCreditAction::LIQUIDATION_RECORD_PAYMENT_RECEIVED,
            )
            .await?;

        let mut db = self.repo.begin_op().await?;
        let mut liquidation = self.repo.find_by_id_in_op(&mut db, liquidation_id).await?;

        let tx_id = CalaTransactionId::new();

        if let Idempotent::Executed(data) = liquidation.record_proceeds_from_liquidation(
            amount_received,
            PaymentId::new(),
            tx_id,
        )? {
            self.repo.update_in_op(&mut db, &mut liquidation).await?;
            self.ledger
                .record_proceeds_from_liquidation_in_op(
                    &mut db,
                    tx_id,
                    data,
                    LedgerTransactionInitiator::try_from_subject(sub)?,
                )
                .await?;
        }

        db.commit().await?;

        Ok(liquidation)
    }

    #[instrument(name = "credit.liquidation.complete_in_op", skip(self, db), err)]
    pub async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        liquidation_id: LiquidationId,
        payment_id: PaymentId,
    ) -> Result<(), LiquidationError> {
        let mut liquidation = self.repo.find_by_id(liquidation_id).await?;

        if liquidation.complete(payment_id).did_execute() {
            self.repo.update_in_op(db, &mut liquidation).await?;
        }

        Ok(())
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
