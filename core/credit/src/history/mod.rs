mod entry;
pub mod error;
mod jobs;
mod repo;

use std::sync::Arc;

use obix::out::{Outbox, OutboxEventMarker};

use audit::AuditSvc;
use authz::PermissionCheck;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditObject, event::CoreCreditEvent, primitives::CreditFacilityId,
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
    pub fn process_event(&mut self, event: &CoreCreditEvent) {
        use CoreCreditEvent::*;

        match event {
            FacilityProposalCreated { .. } => {}
            FacilityProposalConcluded { .. } => {}
            FacilityActivated {
                activation_tx_id,
                activated_at,
                amount,
                ..
            } => {
                self.entries.push(CreditFacilityHistoryEntry::Approved(
                    CreditFacilityApproved {
                        cents: *amount,
                        recorded_at: *activated_at,
                        effective: activated_at.date_naive(),
                        tx_id: *activation_tx_id,
                    },
                ));
            }
            FacilityCollateralUpdated {
                abs_diff,
                recorded_at,
                effective,
                action,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Collateral(CollateralUpdated {
                        satoshis: *abs_diff,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        action: *action,
                        tx_id: *ledger_tx_id,
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
            FacilityPaymentAllocated {
                allocation_id,
                amount,
                recorded_at,
                effective,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Payment(IncrementalPayment {
                        recorded_at: *recorded_at,
                        effective: *effective,
                        cents: *amount,
                        payment_id: *allocation_id,
                    }));
            }
            DisbursalSettled {
                amount,
                recorded_at,
                effective,
                ledger_tx_id,
                ..
            } => {
                self.entries
                    .push(CreditFacilityHistoryEntry::Disbursal(DisbursalExecuted {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        tx_id: *ledger_tx_id,
                    }));
            }
            AccrualPosted {
                amount,
                period,
                ledger_tx_id,
                recorded_at,
                effective,
                ..
            } => {
                self.entries.push(CreditFacilityHistoryEntry::Interest(
                    InterestAccrualsPosted {
                        cents: *amount,
                        recorded_at: *recorded_at,
                        effective: *effective,
                        tx_id: *ledger_tx_id,
                        days: period.days(),
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
            ObligationCreated { .. } => {}
            ObligationDue { .. } => {}
            ObligationOverdue { .. } => {}
            ObligationDefaulted { .. } => {}
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
            ObligationCompleted { .. } => {}
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
        E: OutboxEventMarker<CoreCreditEvent>,
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
