mod entity;
pub mod error;
mod jobs;
pub mod ledger;
mod repo;

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use tracing_macros::record_error_severity;

use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use obix::out::{Outbox, OutboxEventMarker};

use crate::{CreditFacilityPublisher, event::CoreCreditEvent, primitives::*};

use ledger::CollateralLedger;

pub use entity::Collateral;
pub(super) use entity::*;
use jobs::{
    credit_facility_liquidations, liquidation_payment, partial_liquidation, wallet_collateral_sync,
};

#[cfg(feature = "json-schema")]
pub use entity::CollateralEvent;
use error::CollateralError;
use repo::CollateralRepo;

pub struct Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<CollateralRepo<E>>,
    ledger: Arc<CollateralLedger>,
}

impl<Perms, E> Clone for Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms, E> Collaterals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        publisher: &CreditFacilityPublisher<E>,
        ledger: Arc<CollateralLedger>,
        outbox: &Outbox<E>,
        jobs: &mut job::Jobs,
        proceeds_omnibus_account_ids: &crate::LedgerOmnibusAccountIds,
        credit_ledger: Arc<crate::CreditLedger>,
    ) -> Result<Self, CollateralError> {
        let clock = jobs.clock().clone();
        let repo_arc = Arc::new(CollateralRepo::new(pool, publisher, clock.clone()));

        let wallet_collateral_sync_job_spawner =
            jobs.add_initializer(wallet_collateral_sync::WalletCollateralSyncInit::new(
                outbox,
                ledger.clone(),
                repo_arc.clone(),
            ));

        wallet_collateral_sync_job_spawner
            .spawn_unique(
                job::JobId::new(),
                wallet_collateral_sync::WalletCollateralSyncJobConfig::new(),
            )
            .await?;

        let liquidation_repo = Arc::new(crate::liquidation::LiquidationRepo::new(
            pool,
            publisher,
            clock.clone(),
        ));
        let payment_repo = Arc::new(crate::payment::PaymentRepo::new(
            pool,
            publisher,
            clock.clone(),
        ));
        let credit_facility_repo = Arc::new(crate::credit_facility::CreditFacilityRepo::new(
            pool, publisher, clock,
        ));

        let partial_liquidation_job_spawner = jobs.add_initializer(
            partial_liquidation::PartialLiquidationInit::new(outbox, liquidation_repo.clone()),
        );

        let liquidation_payment_job_spawner =
            jobs.add_initializer(liquidation_payment::LiquidationPaymentInit::new(
                outbox,
                payment_repo,
                credit_facility_repo,
                credit_ledger,
            ));

        let credit_facility_liquidations_job_spawner = jobs.add_initializer(
            credit_facility_liquidations::CreditFacilityLiquidationsInit::new(
                outbox,
                liquidation_repo,
                proceeds_omnibus_account_ids,
                partial_liquidation_job_spawner,
                liquidation_payment_job_spawner,
            ),
        );

        credit_facility_liquidations_job_spawner
            .spawn_unique(
                job::JobId::new(),
                credit_facility_liquidations::CreditFacilityLiquidationsJobConfig {
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(Self {
            authz,
            repo: repo_arc,
            ledger,
        })
    }

    pub async fn find_all<T: From<Collateral>>(
        &self,
        ids: &[CollateralId],
    ) -> Result<HashMap<CollateralId, T>, CollateralError> {
        self.repo.find_all(ids).await
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        pending_credit_facility_id: PendingCreditFacilityId,
        custody_wallet_id: Option<CustodyWalletId>,
        account_id: CalaAccountId,
    ) -> Result<Collateral, CollateralError> {
        let new_collateral = NewCollateral::builder()
            .id(collateral_id)
            .credit_facility_id(pending_credit_facility_id)
            .pending_credit_facility_id(pending_credit_facility_id)
            .account_id(account_id)
            .custody_wallet_id(custody_wallet_id)
            .build()
            .expect("all fields for new collateral provided");

        self.repo.create_in_op(db, new_collateral).await
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_manual_input_in_op",
        skip(db, self)
    )]
    pub(super) async fn record_collateral_update_via_manual_input_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        updated_collateral: core_money::Satoshis,
        effective: chrono::NaiveDate,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id(collateral_id).await?;

        if collateral.custody_wallet_id.is_some() {
            return Err(CollateralError::ManualUpdateError);
        }

        let res = if let es_entity::Idempotent::Executed(data) =
            collateral.record_collateral_update_via_manual_input(updated_collateral, effective)
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }

    #[record_error_severity]
    #[instrument(
        name = "collateral.record_collateral_update_via_liquidation_in_op",
        skip(db, self)
    )]
    pub(super) async fn record_collateral_update_via_liquidation_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        collateral_id: CollateralId,
        liquidation_id: LiquidationId,
        amount_sent: core_money::Satoshis,
        effective: chrono::NaiveDate,
        collateral_in_liquidation_account_id: CalaAccountId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<CollateralUpdate>, CollateralError> {
        let mut collateral = self.repo.find_by_id_in_op(&mut *db, collateral_id).await?;

        let res = if let es_entity::Idempotent::Executed(data) = collateral
            .record_collateral_update_via_liquidation(liquidation_id, amount_sent, effective)
        {
            self.repo.update_in_op(db, &mut collateral).await?;
            self.ledger
                .record_collateral_sent_to_liquidation_in_op(
                    db,
                    data.tx_id,
                    amount_sent,
                    collateral.account_id,
                    collateral_in_liquidation_account_id,
                    initiated_by,
                )
                .await?;
            Some(data)
        } else {
            None
        };

        Ok(res)
    }
}
