mod entity;
pub mod error;
mod ledger;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{
    AccountId as CalaAccountId, CalaLedger, JournalId, TransactionId as CalaTransactionId,
};
use core_money::{Satoshis, UsdCents};
use es_entity::{DbOp, Idempotent};
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, CreditFacilityId, LedgerOmnibusAccountIds,
    LiquidationId, PaymentId,
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
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: LiquidationRepo<E>,
    authz: Arc<Perms>,
    ledger: LiquidationLedger,
    omnibus_account_ids: LedgerOmnibusAccountIds,
}

impl<Perms, E> Clone for Liquidations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
            omnibus_account_ids: self.omnibus_account_ids.clone(),
        }
    }
}

impl<Perms, E> Liquidations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        journal_id: JournalId,
        cala: &CalaLedger,
        omnibus_account_ids: &LedgerOmnibusAccountIds,
        authz: Arc<Perms>,
        publisher: &crate::CreditFacilityPublisher<E>,
    ) -> Result<Self, LiquidationError> {
        Ok(Self {
            repo: LiquidationRepo::new(pool, publisher),
            authz,
            ledger: LiquidationLedger::init(cala, journal_id).await?,
            omnibus_account_ids: omnibus_account_ids.clone(),
        })
    }

    #[instrument(
        name = "credit.liquidation.create_if_not_exist_for_facility_in_op",
        skip(self, db, new_liquidation),
        fields(existing_liquidation_found),
        err
    )]
    pub async fn create_if_not_exist_for_facility_in_op(
        &self,
        db: &mut DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        new_liquidation: &mut NewLiquidationBuilder,
    ) -> Result<Option<Liquidation>, LiquidationError> {
        let existing_liquidation = self
            .repo
            .maybe_find_active_liquidation_for_credit_facility_id_in_op(
                &mut *db,
                credit_facility_id,
            )
            .await?;

        tracing::Span::current()
            .record("existing_liquidation_found", existing_liquidation.is_some());

        if existing_liquidation.is_none() {
            let liquidation = self
                .repo
                .create_in_op(
                    db,
                    new_liquidation
                        .omnibus_account_id(self.omnibus_account_ids.account_id)
                        .build()
                        .expect("Could not build new liquidation"),
                )
                .await?;
            Ok(Some(liquidation))
        } else {
            Ok(None)
        }
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
                )
                .await?;
        }

        db.commit().await?;

        Ok(liquidation)
    }

    #[instrument(
        name = "credit.liquidation.record_payment_from_liquidation",
        skip(self, sub),
        err
    )]
    pub async fn record_payment_from_liquidation(
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

        let mut liquidation = self.repo.find_by_id(liquidation_id).await?;
        let mut db = self.repo.begin_op().await?;

        let tx_id = CalaTransactionId::new();

        if let Idempotent::Executed(data) = liquidation.record_repayment_from_liquidation(
            amount_received,
            PaymentId::new(),
            tx_id,
        )? {
            self.repo.update_in_op(&mut db, &mut liquidation).await?;
            self.ledger
                .record_payment_from_liquidation_in_op(&mut db, tx_id, data)
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
pub struct RecordPaymentFromLiquidationData {
    pub omnibus_account_id: CalaAccountId,
    pub facility_holding_account_id: CalaAccountId,
    pub amount_received: UsdCents,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
    pub amount_liquidated: Satoshis,
}
