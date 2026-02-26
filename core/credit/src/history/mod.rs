mod entry;
pub mod error;
mod jobs;
mod repo;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    collateral::public::CoreCreditCollateralEvent, primitives::CreditFacilityId,
};
pub use entry::*;
use error::CreditFacilityHistoryError;
use jobs::{
    credit_facility_history, update_collateral_history, update_collection_history,
    update_credit_history,
};
use repo::HistoryRepo;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct CreditFacilityHistory {
    entries: Vec<CreditFacilityHistoryEntry>,
}

impl IntoIterator for CreditFacilityHistory {
    type Item = CreditFacilityHistoryEntry;
    type IntoIter = std::iter::Rev<std::vec::IntoIter<CreditFacilityHistoryEntry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter().rev()
    }
}

impl CreditFacilityHistory {
    pub fn process_credit_event(
        &mut self,
        event: &CoreCreditEvent,
        message_recorded_at: DateTime<Utc>,
    ) {
        use CoreCreditEvent::*;

        match event {
            FacilityProposalCreated { .. } => {}
            FacilityProposalConcluded { .. } => {}
            FacilityActivated { entity } => {
                self.entries.push(CreditFacilityHistoryEntry::Approved(
                    CreditFacilityApproved {
                        cents: entity.amount,
                        recorded_at: entity.activated_at,
                        effective: entity.activated_at.date_naive(),
                        tx_id: entity.activation_tx_id,
                    },
                ));
            }
            FacilityCollateralizationChanged { entity } => {
                let collateralization = &entity.collateralization;
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateralization(
                        CollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral,
                            outstanding_interest: collateralization.outstanding.interest,
                            outstanding_disbursal: collateralization.outstanding.disbursed,
                            recorded_at: message_recorded_at,
                            effective: message_recorded_at.date_naive(),
                            price: collateralization.price_at_state_change,
                        },
                    ));
            }
            DisbursalSettled { entity } => {
                let settlement = entity
                    .settlement
                    .as_ref()
                    .expect("settlement must be set for DisbursalSettled");
                self.entries
                    .push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                        cents: entity.amount,
                        recorded_at: message_recorded_at,
                        effective: settlement.effective,
                        tx_id: settlement.tx_id,
                    }));
            }
            AccrualPosted { entity } => {
                let posting = entity
                    .posting
                    .as_ref()
                    .expect("posting must be set for AccrualPosted");
                self.entries.push(CreditFacilityHistoryEntry::Interest(
                    InterestAccrualsPosted {
                        cents: posting.amount,
                        recorded_at: message_recorded_at,
                        effective: posting.effective,
                        tx_id: posting.tx_id,
                        days: entity.period.days(),
                    },
                ));
            }
            PendingCreditFacilityCollateralizationChanged { entity } => {
                let collateralization = &entity.collateralization;
                self.entries.push(
                    CreditFacilityHistoryEntry::PendingCreditFacilityCollateralization(
                        PendingCreditFacilityCollateralizationUpdated {
                            state: collateralization.state,
                            collateral: collateralization.collateral.expect("collateral must be set for PendingCreditFacilityCollateralizationChanged"),
                            recorded_at: message_recorded_at,
                            effective: message_recorded_at.date_naive(),
                            price: collateralization.price_at_state_change.expect("price must be set for PendingCreditFacilityCollateralizationChanged"),
                        },
                    ),
                );
            }
            PendingCreditFacilityCompleted { .. } => {}
            FacilityCompleted { .. } => {}
            PartialLiquidationInitiated { .. } => {}
        }
    }

    pub fn process_collateral_event(
        &mut self,
        event: &CoreCreditCollateralEvent,
        message_recorded_at: DateTime<Utc>,
    ) {
        match event {
            CoreCreditCollateralEvent::CollateralUpdated { entity } => {
                let adjustment = entity
                    .adjustment
                    .as_ref()
                    .expect("adjustment must be set for FacilityCollateralUpdated");
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: adjustment.abs_diff,
                        recorded_at: message_recorded_at,
                        effective: message_recorded_at.date_naive(),
                        direction: adjustment.direction,
                        tx_id: adjustment.tx_id,
                    }));
            }
            CoreCreditCollateralEvent::LiquidationCollateralSentOut {
                amount,
                recorded_at,
                effective,
                ledger_tx_id,
                ..
            } => self
                .entries
                .push(CreditFacilityHistoryEntry::Liquidation(CollateralSentOut {
                    amount: *amount,
                    recorded_at: *recorded_at,
                    effective: *effective,
                    tx_id: *ledger_tx_id,
                })),
            CoreCreditCollateralEvent::LiquidationProceedsReceived {
                amount,
                recorded_at,
                effective,
                ledger_tx_id,
                ..
            } => self.entries.push(CreditFacilityHistoryEntry::Repayment(
                ProceedsFromLiquidationReceived {
                    cents: *amount,
                    recorded_at: *recorded_at,
                    effective: *effective,
                    tx_id: *ledger_tx_id,
                },
            )),
            CoreCreditCollateralEvent::LiquidationCompleted { .. } => {}
        }
    }

    pub fn process_collection_event(&mut self, event: &CoreCreditCollectionEvent) {
        if let CoreCreditCollectionEvent::PaymentAllocationCreated { entity } = event {
            self.entries
                .push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                    recorded_at: entity.recorded_at,
                    effective: entity.effective,
                    cents: entity.amount,
                    payment_id: entity.id,
                }));
        }
    }
}

pub struct Histories<Perms>
where
    Perms: PermissionCheck,
{
    repo: Arc<HistoryRepo>,
    authz: Arc<Perms>,
}

impl<Perms> Clone for Histories<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> Histories<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub async fn init<E>(
        pool: &sqlx::PgPool,
        outbox: &Outbox<E>,
        job: &mut job::Jobs,
        authz: Arc<Perms>,
    ) -> Result<Self, error::CreditFacilityHistoryError>
    where
        E: OutboxEventMarker<CoreCreditEvent>
            + OutboxEventMarker<CoreCreditCollateralEvent>
            + OutboxEventMarker<crate::CoreCreditCollectionEvent>,
    {
        let repo = Arc::new(HistoryRepo::new(pool));

        let update_credit_history = job.add_initializer(
            update_credit_history::UpdateCreditHistoryJobInitializer::new(repo.clone()),
        );
        let update_collateral_history = job.add_initializer(
            update_collateral_history::UpdateCollateralHistoryJobInitializer::new(repo.clone()),
        );
        let update_collection_history = job.add_initializer(
            update_collection_history::UpdateCollectionHistoryJobInitializer::new(repo.clone()),
        );
        outbox
            .register_event_handler(
                job,
                OutboxEventJobConfig::new(credit_facility_history::HISTORY_PROJECTION),
                credit_facility_history::HistoryProjectionHandler::new(
                    update_credit_history,
                    update_collateral_history,
                    update_collection_history,
                ),
            )
            .await?;

        Ok(Self { repo, authz })
    }

    #[record_error_severity]
    #[instrument(name = "credit.history", skip(self, credit_facility_id), fields(credit_facility_id = tracing::field::Empty))]
    pub async fn find_for_credit_facility_id<T: From<CreditFacilityHistoryEntry>>(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CreditFacilityHistoryError> {
        let id = credit_facility_id.into();
        tracing::Span::current().record("credit_facility_id", tracing::field::display(id));

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        let history = self.repo.load(id).await?;
        Ok(history.into_iter().map(T::from).collect())
    }

    pub(crate) async fn find_for_credit_facility_id_without_audit(
        &self,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<CreditFacilityHistoryEntry>, CreditFacilityHistoryError> {
        let history = self.repo.load(credit_facility_id.into()).await?;
        Ok(history.into_iter().collect())
    }
}
