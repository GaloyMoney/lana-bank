mod entry;
pub mod error;
mod jobs;
mod repo;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use obix::out::{Outbox, OutboxEventMarker};

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    primitives::CreditFacilityId,
};
pub use entry::*;
use error::CreditFacilityHistoryError;
use jobs::*;
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
    pub fn process_credit_event(&mut self, event: &CoreCreditEvent, recorded_at: DateTime<Utc>) {
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
            FacilityCollateralUpdated { entity } => {
                let adjustment = entity
                    .adjustment
                    .as_ref()
                    .expect("adjustment must be set for FacilityCollateralUpdated");
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: adjustment.abs_diff,
                        recorded_at,
                        effective: recorded_at.date_naive(),
                        direction: adjustment.direction,
                        tx_id: adjustment.tx_id,
                    }));
            }
            FacilityCollateralizationChanged {
                state,
                recorded_at,
                effective,
                outstanding,
                price,
                collateral,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateralization(
                        CollateralizationUpdated {
                            state: *state,
                            collateral: *collateral,
                            outstanding_interest: outstanding.interest,
                            outstanding_disbursal: outstanding.disbursed,
                            recorded_at: *recorded_at,
                            effective: *effective,
                            price: *price,
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
                        recorded_at,
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
                        recorded_at,
                        effective: posting.effective,
                        tx_id: posting.tx_id,
                        days: entity.period.days(),
                    },
                ));
            }
            PendingCreditFacilityCollateralizationChanged {
                state,
                collateral,
                price,
                recorded_at,
                effective,
                ..
            } => self.entries.push(
                CreditFacilityHistoryEntry::PendingCreditFacilityCollateralization(
                    PendingCreditFacilityCollateralizationUpdated {
                        state: *state,
                        collateral: *collateral,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        price: *price,
                    },
                ),
            ),
            PendingCreditFacilityCompleted { .. } => {}
            FacilityCompleted { .. } => {}
            PartialLiquidationInitiated { .. } => {}
            PartialLiquidationCollateralSentOut {
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
            PartialLiquidationProceedsReceived {
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
            PartialLiquidationCompleted { .. } => {}
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
        E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<crate::CoreCreditCollectionEvent>,
    {
        let repo = Arc::new(HistoryRepo::new(pool));

        let job_init = credit_facility_history::HistoryProjectionInit::new(outbox, repo.clone());

        let spawner = job.add_initializer(job_init);

        spawner
            .spawn_unique(
                job::JobId::new(),
                credit_facility_history::HistoryProjectionConfig {
                    _phantom: std::marker::PhantomData,
                },
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
