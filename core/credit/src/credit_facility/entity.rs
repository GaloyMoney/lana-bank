use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    obligation::{NewObligation, ObligationType, ObligationsAmounts},
    primitives::*,
    terms::{CVLPct, CollateralizationState, InterestPeriod, TermValues},
};

use crate::{interest_accrual_cycle::*, ledger::*};

use super::{
    balance::CreditFacilityBalanceSummary, cvl::*, error::CreditFacilityError, history,
    repayment_plan,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BalanceUpdatedSource {
    Obligation(ObligationId),
    PaymentAllocation(LedgerTxId), // TODO: change to PaymentAllocationId
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BalanceUpdatedType {
    Disbursal,
    InterestAccrual,
}

impl From<ObligationType> for BalanceUpdatedType {
    fn from(obligation_type: ObligationType) -> Self {
        match obligation_type {
            ObligationType::Disbursal => Self::Disbursal,
            ObligationType::Interest => Self::InterestAccrual,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityId")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        terms: Box<TermValues>,
        amount: UsdCents,
        account_ids: CreditFacilityAccountIds,
        disbursal_credit_account_id: CalaAccountId,
        approval_process_id: ApprovalProcessId,
        audit_info: AuditInfo,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
        audit_info: AuditInfo,
    },
    Activated {
        ledger_tx_id: LedgerTxId,
        activated_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    BalanceUpdated {
        source: BalanceUpdatedSource,
        balance_type: BalanceUpdatedType,
        amount: UsdCents,
        updated_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrualCycleStarted {
        interest_accrual_id: InterestAccrualCycleId,
        idx: InterestAccrualCycleIdx,
        started_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrualCycleConcluded {
        idx: InterestAccrualCycleIdx,
        tx_id: LedgerTxId,
        obligation_id: ObligationId,
        audit_info: AuditInfo,
    },
    CollateralUpdated {
        tx_id: LedgerTxId,
        total_collateral: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        recorded_in_ledger_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    CollateralizationChanged {
        state: CollateralizationState,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        price: PriceOfOneBTC,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    Completed {
        completed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditFacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl From<CreditFacilityBalanceSummary> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityBalanceSummary) -> Self {
        Self {
            disbursed: balance.disbursed_outstanding(),
            interest: balance.interest_outstanding(),
        }
    }
}

impl From<ObligationsAmounts> for CreditFacilityReceivable {
    fn from(outstanding: ObligationsAmounts) -> Self {
        Self {
            disbursed: outstanding.disbursed,
            interest: outstanding.interest,
        }
    }
}

impl CreditFacilityReceivable {
    pub fn total(&self) -> UsdCents {
        self.interest + self.disbursed
    }

    pub fn disbursed_cvl(&self, collateral: Satoshis) -> CVLData {
        CVLData::new(collateral, self.total())
    }

    pub fn total_cvl(&self, collateral: Satoshis, facility_remaining: UsdCents) -> CVLData {
        CVLData::new(collateral, self.total() + facility_remaining)
    }

    pub fn is_zero(&self) -> bool {
        self.total().is_zero()
    }

    pub fn with_added_disbursal_amount(&self, amount: UsdCents) -> Self {
        Self {
            disbursed: self.disbursed + amount,
            interest: self.interest,
        }
    }

    pub fn facility_cvl_data(
        &self,
        collateral: Satoshis,
        facility_remaining: UsdCents,
    ) -> FacilityCVLData {
        FacilityCVLData {
            total: self.total_cvl(collateral, facility_remaining),
            disbursed: self.disbursed_cvl(collateral),
        }
    }
}

#[derive(Debug)]
pub(crate) struct NewAccrualPeriods {
    pub(crate) accrual: InterestPeriod,
    pub(super) _accrual_cycle: InterestPeriod,
}

impl From<(InterestAccrualData, CreditFacilityAccountIds)> for CreditFacilityInterestAccrual {
    fn from(data: (InterestAccrualData, CreditFacilityAccountIds)) -> Self {
        let (
            InterestAccrualData {
                interest,
                period,
                tx_ref,
                tx_id,
            },
            credit_facility_account_ids,
        ) = data;
        Self {
            interest,
            period,
            tx_ref,
            tx_id,
            credit_facility_account_ids,
        }
    }
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct CreditFacility {
    pub id: CreditFacilityId,
    pub approval_process_id: ApprovalProcessId,
    pub customer_id: CustomerId,
    pub amount: UsdCents,
    pub terms: TermValues,
    pub account_ids: CreditFacilityAccountIds,
    pub disbursal_credit_account_id: CalaAccountId,
    #[builder(setter(strip_option), default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub matures_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub defaults_at: Option<DateTime<Utc>>,

    #[es_entity(nested)]
    #[builder(default)]
    interest_accruals: Nested<InterestAccrualCycle>,
    events: EntityEvents<CreditFacilityEvent>,
}

impl CreditFacility {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn activated_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter_all().find_map(|event| match event {
            CreditFacilityEvent::Activated { activated_at, .. } => Some(*activated_at),
            _ => None,
        })
    }

    pub fn structuring_fee(&self) -> UsdCents {
        self.terms.one_time_fee_rate.apply(self.amount)
    }

    fn facility_remaining(&self, amount_disbursed: UsdCents) -> UsdCents {
        self.amount - amount_disbursed
    }

    pub fn history(&self) -> Vec<history::CreditFacilityHistoryEntry> {
        history::project(self.events.iter_all())
    }

    pub fn repayment_plan(&self) -> Vec<repayment_plan::CreditFacilityRepaymentInPlan> {
        repayment_plan::project(self.events.iter_all())
    }

    pub(crate) fn is_approval_process_concluded(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::ApprovalProcessConcluded { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    fn is_approved(&self) -> Result<bool, CreditFacilityError> {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::ApprovalProcessConcluded { approved, .. } => {
                    return Ok(*approved)
                }
                _ => continue,
            }
        }
        Err(CreditFacilityError::ApprovalInProgress)
    }

    fn is_activated(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::Activated { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn is_after_maturity_date(&self) -> bool {
        let now = crate::time::now();
        self.matures_at.is_some_and(|matures_at| now > matures_at)
    }

    pub fn status(&self) -> CreditFacilityStatus {
        if self.is_completed() {
            CreditFacilityStatus::Closed
        } else if self.is_after_maturity_date() {
            CreditFacilityStatus::Matured
        } else if self.is_activated() {
            CreditFacilityStatus::Active
        } else if self.is_fully_collateralized() {
            CreditFacilityStatus::PendingApproval
        } else {
            CreditFacilityStatus::PendingCollateralization
        }
    }

    pub(crate) fn approval_process_concluded(
        &mut self,
        approved: bool,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityEvent::ApprovalProcessConcluded { .. }
        );
        self.events
            .push(CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: self.id.into(),
                approved,
                audit_info,
            });
        Idempotent::Executed(())
    }

    pub(crate) fn activate(
        &mut self,
        activated_at: DateTime<Utc>,
        price: PriceOfOneBTC,
        balances: CreditFacilityBalanceSummary,
        audit_info: AuditInfo,
    ) -> Result<Idempotent<(CreditFacilityActivation, InterestPeriod)>, CreditFacilityError> {
        if self.is_activated() {
            return Ok(Idempotent::Ignored);
        }

        if !self.is_approval_process_concluded() {
            return Err(CreditFacilityError::ApprovalInProgress);
        }

        if !self.is_approved()? {
            return Err(CreditFacilityError::Denied);
        }

        if self.collateral() == Satoshis::ZERO {
            return Err(CreditFacilityError::NoCollateral);
        }

        self.facility_cvl_data(balances)
            .cvl(price)
            .check_approval_allowed(self.terms)?;

        self.activated_at = Some(activated_at);
        self.matures_at = Some(self.terms.duration.maturity_date(activated_at));
        self.defaults_at = self
            .terms
            .interest_overdue_duration
            .map(|d| d.end_date(self.matures_at.expect("No 'matures_at' date set")));
        let tx_id = LedgerTxId::new();
        self.events.push(CreditFacilityEvent::Activated {
            ledger_tx_id: tx_id,
            activated_at,
            audit_info: audit_info.clone(),
        });

        let periods = self
            .start_interest_accrual_cycle(audit_info)
            .expect("first accrual")
            .expect("first accrual");
        let activation = CreditFacilityActivation {
            tx_id,
            tx_ref: format!("{}-activate", self.id),
            credit_facility_account_ids: self.account_ids,
            debit_account_id: self.disbursal_credit_account_id,
            facility_amount: self.amount,
            structuring_fee_amount: self.structuring_fee(),
        };

        Ok(Idempotent::Executed((activation, periods.accrual)))
    }

    pub(crate) fn can_initiate_disbursal(&self, initiated_at: DateTime<Utc>) -> bool {
        initiated_at < self.matures_at.expect("Facility not activated yet")
    }

    fn next_interest_accrual_cycle_period(
        &self,
    ) -> Result<Option<InterestPeriod>, CreditFacilityError> {
        let last_accrual_start_date = self.events.iter_all().rev().find_map(|event| match event {
            CreditFacilityEvent::InterestAccrualCycleStarted { started_at, .. } => {
                Some(*started_at)
            }
            _ => None,
        });

        let interval = self.terms.accrual_cycle_interval;
        let full_period = match last_accrual_start_date {
            Some(last_accrual_start_date) => interval.period_from(last_accrual_start_date).next(),
            None => interval.period_from(
                self.activated_at()
                    .ok_or(CreditFacilityError::NotActivatedYet)?,
            ),
        };

        Ok(full_period.truncate(self.matures_at.expect("Facility is already active")))
    }

    pub(crate) fn start_interest_accrual_cycle(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Option<NewAccrualPeriods>, CreditFacilityError> {
        let accrual_cycle_period = match self.next_interest_accrual_cycle_period()? {
            Some(period) => period,
            None => return Ok(None),
        };
        let now = crate::time::now();
        if accrual_cycle_period.start > now {
            return Err(CreditFacilityError::InterestAccrualCycleWithInvalidFutureStartDate);
        }

        let idx = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualCycleStarted { idx, .. } => Some(idx.next()),
                _ => None,
            })
            .unwrap_or(InterestAccrualCycleIdx::FIRST);
        let id = InterestAccrualCycleId::new();
        self.events
            .push(CreditFacilityEvent::InterestAccrualCycleStarted {
                interest_accrual_id: id,
                idx,
                started_at: accrual_cycle_period.start,
                audit_info: audit_info.clone(),
            });

        let new_accrual = NewInterestAccrualCycle::builder()
            .id(id)
            .credit_facility_id(self.id)
            .account_ids(self.account_ids.into())
            .idx(idx)
            .started_at(accrual_cycle_period.start)
            .facility_matures_at(self.matures_at.expect("Facility is already approved"))
            .terms(self.terms)
            .audit_info(audit_info)
            .build()
            .expect("could not build new interest accrual");
        Ok(Some(NewAccrualPeriods {
            accrual: self
                .interest_accruals
                .add_new(new_accrual)
                .first_accrual_cycle_period(),
            _accrual_cycle: accrual_cycle_period,
        }))
    }

    pub(crate) fn record_interest_accrual_cycle(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<NewObligation, CreditFacilityError> {
        let accrual_cycle_data = self
            .interest_accrual_cycle_in_progress()
            .expect("accrual not found")
            .accrual_cycle_data()
            .ok_or(CreditFacilityError::InterestAccrualNotCompletedYet)?;

        let (idx, new_obligation) = {
            let accrual = self
                .interest_accrual_cycle_in_progress_mut()
                .expect("accrual not found");
            (
                accrual.idx,
                accrual.record_accrual_cycle(accrual_cycle_data.clone(), audit_info.clone()),
            )
        };
        self.events
            .push(CreditFacilityEvent::InterestAccrualCycleConcluded {
                idx,
                obligation_id: new_obligation.id(),
                tx_id: accrual_cycle_data.tx_id,
                audit_info: audit_info.clone(),
            });
        self.events.push(CreditFacilityEvent::BalanceUpdated {
            source: BalanceUpdatedSource::Obligation(new_obligation.id()),
            balance_type: BalanceUpdatedType::InterestAccrual,
            amount: accrual_cycle_data.interest,
            updated_at: accrual_cycle_data.posted_at,
            audit_info,
        });

        Ok(new_obligation)
    }

    pub fn interest_accrual_cycle_in_progress(&self) -> Option<&InterestAccrualCycle> {
        if let Some(id) = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualCycleConcluded { .. } => Some(None),
                CreditFacilityEvent::InterestAccrualCycleStarted {
                    interest_accrual_id: id,
                    ..
                } => Some(Some(id)),
                _ => None,
            })
            .flatten()
        {
            Some(
                self.interest_accruals
                    .get_persisted(id)
                    .expect("Interest accrual not found"),
            )
        } else {
            None
        }
    }

    pub fn interest_accrual_cycle_in_progress_mut(&mut self) -> Option<&mut InterestAccrualCycle> {
        if let Some(id) = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualCycleConcluded { .. } => Some(None),
                CreditFacilityEvent::InterestAccrualCycleStarted {
                    interest_accrual_id: id,
                    ..
                } => Some(Some(id)),
                _ => None,
            })
            .flatten()
        {
            Some(
                self.interest_accruals
                    .get_persisted_mut(id)
                    .expect("Interest accrual not found"),
            )
        } else {
            None
        }
    }

    pub(crate) fn update_balance_from_payment(
        &mut self,
        payment_allocation_id: LedgerTxId,
        balance_type: impl Into<BalanceUpdatedType>,
        amount: UsdCents,
        updated_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CreditFacilityEvent::BalanceUpdated {
                source,
                ..
            } if *source == BalanceUpdatedSource::PaymentAllocation(payment_allocation_id)
        );

        self.events.push(CreditFacilityEvent::BalanceUpdated {
            source: BalanceUpdatedSource::PaymentAllocation(payment_allocation_id),
            balance_type: balance_type.into(),
            amount,
            updated_at,
            audit_info,
        });

        Idempotent::Executed(())
    }

    pub fn collateral(&self) -> Satoshis {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::CollateralUpdated {
                    total_collateral, ..
                } => Some(*total_collateral),
                _ => None,
            })
            .unwrap_or(Satoshis::ZERO)
    }

    pub fn facility_cvl_data(&self, balances: CreditFacilityBalanceSummary) -> FacilityCVLData {
        CreditFacilityReceivable::from(balances).facility_cvl_data(
            self.collateral(),
            self.facility_remaining(balances.disbursed),
        )
    }

    pub fn last_collateralization_state(&self) -> CollateralizationState {
        if self.is_completed() {
            return CollateralizationState::NoCollateral;
        }

        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::CollateralizationChanged { state, .. } => Some(*state),
                _ => None,
            })
            .unwrap_or(CollateralizationState::NoCollateral)
    }

    fn is_fully_collateralized(&self) -> bool {
        self.last_collateralization_state() == CollateralizationState::FullyCollateralized
    }

    pub(crate) fn maybe_update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
        audit_info: &AuditInfo,
    ) -> Option<CollateralizationState> {
        let facility_cvl = self.facility_cvl_data(balances).cvl(price);
        let last_collateralization_state = self.last_collateralization_state();

        let collateralization_update =
            match self.status() {
                CreditFacilityStatus::PendingCollateralization
                | CreditFacilityStatus::PendingApproval => facility_cvl
                    .total
                    .collateralization_update(self.terms, last_collateralization_state, None, true),
                CreditFacilityStatus::Active | CreditFacilityStatus::Matured => {
                    let cvl = if balances.any_disbursed() {
                        facility_cvl.disbursed
                    } else {
                        facility_cvl.total
                    };

                    cvl.collateralization_update(
                        self.terms,
                        last_collateralization_state,
                        Some(upgrade_buffer_cvl_pct),
                        false,
                    )
                }
                CreditFacilityStatus::Closed => Some(CollateralizationState::NoCollateral),
            };

        let now = crate::time::now();
        if let Some(calculated_collateralization) = collateralization_update {
            self.events
                .push(CreditFacilityEvent::CollateralizationChanged {
                    state: calculated_collateralization,
                    collateral: self.collateral(),
                    outstanding: balances.into(),
                    price,
                    recorded_at: now,
                    audit_info: audit_info.clone(),
                });

            return Some(calculated_collateralization);
        }

        None
    }

    pub(crate) fn record_collateral_update(
        &mut self,
        updated_collateral: Satoshis,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
    ) -> Result<CreditFacilityCollateralUpdate, CreditFacilityError> {
        let current_collateral = self.collateral();
        let diff =
            SignedSatoshis::from(updated_collateral) - SignedSatoshis::from(current_collateral);

        if diff == SignedSatoshis::ZERO {
            return Err(CreditFacilityError::CollateralNotUpdated(
                current_collateral,
                updated_collateral,
            ));
        }

        let (collateral, action) = if diff > SignedSatoshis::ZERO {
            (Satoshis::try_from(diff)?, CollateralAction::Add)
        } else {
            (Satoshis::try_from(diff.abs())?, CollateralAction::Remove)
        };

        let collateral_update = CreditFacilityCollateralUpdate {
            abs_diff: collateral,
            credit_facility_account_ids: self.account_ids,
            tx_id: LedgerTxId::new(),
            action,
        };
        self.confirm_collateral_update(
            collateral_update.clone(),
            crate::time::now(),
            audit_info,
            price,
            upgrade_buffer_cvl_pct,
            balances,
        );

        Ok(collateral_update)
    }

    fn confirm_collateral_update(
        &mut self,
        CreditFacilityCollateralUpdate {
            tx_id,
            abs_diff,
            action,
            ..
        }: CreditFacilityCollateralUpdate,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
    ) {
        let mut total_collateral = self.collateral();
        total_collateral = match action {
            CollateralAction::Add => total_collateral + abs_diff,
            CollateralAction::Remove => total_collateral - abs_diff,
        };
        self.events.push(CreditFacilityEvent::CollateralUpdated {
            tx_id,
            total_collateral,
            abs_diff,
            action,
            recorded_in_ledger_at: executed_at,
            audit_info: audit_info.clone(),
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, balances, &audit_info);
    }

    pub(crate) fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .any(|event| matches!(event, CreditFacilityEvent::Completed { .. }))
    }

    pub(crate) fn complete(
        &mut self,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
    ) -> Result<Idempotent<CreditFacilityCompletion>, CreditFacilityError> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityEvent::Completed { .. }
        );
        if balances.any_outstanding() {
            return Err(CreditFacilityError::OutstandingAmount);
        }

        let res = CreditFacilityCompletion {
            tx_id: LedgerTxId::new(),
            collateral: self.collateral(),
            credit_facility_account_ids: self.account_ids,
        };

        let completed_at = crate::time::now();
        self.confirm_collateral_update(
            CreditFacilityCollateralUpdate {
                credit_facility_account_ids: self.account_ids,
                tx_id: res.tx_id,
                abs_diff: res.collateral,
                action: CollateralAction::Remove,
            },
            completed_at,
            audit_info.clone(),
            price,
            upgrade_buffer_cvl_pct,
            balances,
        );

        self.events.push(CreditFacilityEvent::Completed {
            completed_at,
            audit_info,
        });

        Ok(Idempotent::Executed(res))
    }

    fn balance_outstanding(&self) -> UsdCents {
        self.events
            .iter_all()
            .fold(UsdCents::from(0), |mut total, event| {
                if let CreditFacilityEvent::BalanceUpdated { source, amount, .. } = event {
                    match source {
                        BalanceUpdatedSource::Obligation(_) => total += *amount,
                        BalanceUpdatedSource::PaymentAllocation(_) => total -= *amount,
                    }
                }
                total
            })
    }

    pub(super) fn collateralization_ratio(&self) -> Option<Decimal> {
        let amount = if self.status() == CreditFacilityStatus::PendingCollateralization
            || self.status() == CreditFacilityStatus::PendingApproval
        {
            self.amount
        } else {
            self.balance_outstanding()
        };

        if amount > UsdCents::ZERO {
            Some(
                rust_decimal::Decimal::from(self.collateral().into_inner())
                    / Decimal::from(amount.into_inner()),
            )
        } else {
            None
        }
    }
}

impl TryFromEvents<CreditFacilityEvent> for CreditFacility {
    fn try_from_events(events: EntityEvents<CreditFacilityEvent>) -> Result<Self, EsEntityError> {
        let mut builder = CreditFacilityBuilder::default();
        let mut terms = None;
        for event in events.iter_all() {
            match event {
                CreditFacilityEvent::Initialized {
                    id,
                    amount,
                    customer_id,
                    account_ids,
                    disbursal_credit_account_id,
                    terms: t,
                    approval_process_id,
                    ..
                } => {
                    terms = Some(**t);
                    builder = builder
                        .id(*id)
                        .amount(*amount)
                        .customer_id(*customer_id)
                        .terms(**t)
                        .account_ids(*account_ids)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .approval_process_id(*approval_process_id)
                }
                CreditFacilityEvent::Activated { activated_at, .. } => {
                    let matures_at = terms
                        .expect("terms should be set")
                        .duration
                        .maturity_date(*activated_at);
                    let defaults_at = terms
                        .expect("terms should be set")
                        .interest_overdue_duration
                        .map(|d| d.end_date(matures_at));
                    builder = builder
                        .activated_at(*activated_at)
                        .matures_at(matures_at)
                        .defaults_at(defaults_at)
                }
                CreditFacilityEvent::ApprovalProcessConcluded { .. } => (),
                CreditFacilityEvent::BalanceUpdated { .. } => (),
                CreditFacilityEvent::InterestAccrualCycleStarted { .. } => (),
                CreditFacilityEvent::InterestAccrualCycleConcluded { .. } => (),
                CreditFacilityEvent::CollateralUpdated { .. } => (),
                CreditFacilityEvent::CollateralizationChanged { .. } => (),
                CreditFacilityEvent::Completed { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewCreditFacility {
    #[builder(setter(into))]
    pub(super) id: CreditFacilityId,
    #[builder(setter(into))]
    pub(super) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    terms: TermValues,
    amount: UsdCents,
    #[builder(setter(skip), default)]
    pub(super) status: CreditFacilityStatus,
    #[builder(setter(skip), default)]
    pub(super) collateralization_state: CollateralizationState,
    account_ids: CreditFacilityAccountIds,
    disbursal_credit_account_id: CalaAccountId,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewCreditFacility {
    pub fn builder() -> NewCreditFacilityBuilder {
        NewCreditFacilityBuilder::default()
    }
}

impl IntoEvents<CreditFacilityEvent> for NewCreditFacility {
    fn into_events(self) -> EntityEvents<CreditFacilityEvent> {
        EntityEvents::init(
            self.id,
            [CreditFacilityEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info.clone(),
                customer_id: self.customer_id,
                terms: Box::new(self.terms),
                amount: self.amount,
                account_ids: self.account_ids,
                disbursal_credit_account_id: self.disbursal_credit_account_id,
                approval_process_id: self.approval_process_id,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use audit::{AuditEntryId, AuditInfo};
    use rust_decimal_macros::dec;

    use crate::{
        terms::{Duration, InterestInterval, OneTimeFeeRatePct},
        *,
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .interest_due_duration(InterestDuration::Days(0))
            .accrual_cycle_interval(InterestInterval::EndOfMonth)
            .accrual_interval(InterestInterval::EndOfDay)
            .one_time_fee_rate(OneTimeFeeRatePct::new(5))
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn default_facility() -> UsdCents {
        UsdCents::from(10_00)
    }

    fn default_full_collateral() -> Satoshis {
        Satoshis::from(100_000)
    }

    fn default_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::from(5000000))
    }

    fn default_upgrade_buffer_cvl_pct() -> CVLPct {
        CVLPct::new(5)
    }

    fn default_balances(facility_remaining: UsdCents) -> CreditFacilityBalanceSummary {
        CreditFacilityBalanceSummary {
            facility_remaining,
            collateral: Satoshis::ZERO,
            disbursed: UsdCents::ZERO,
            not_yet_due_disbursed_outstanding: UsdCents::ZERO,
            due_disbursed_outstanding: UsdCents::ZERO,
            overdue_disbursed_outstanding: UsdCents::ZERO,
            disbursed_defaulted: UsdCents::ZERO,
            interest_posted: UsdCents::ZERO,
            not_yet_due_interest_outstanding: UsdCents::ZERO,
            due_interest_outstanding: UsdCents::ZERO,
            overdue_interest_outstanding: UsdCents::ZERO,
            interest_defaulted: UsdCents::ZERO,
        }
    }

    fn facility_from(events: Vec<CreditFacilityEvent>) -> CreditFacility {
        CreditFacility::try_from_events(EntityEvents::init(CreditFacilityId::new(), events))
            .unwrap()
    }

    fn initial_events() -> Vec<CreditFacilityEvent> {
        vec![CreditFacilityEvent::Initialized {
            id: CreditFacilityId::new(),
            audit_info: dummy_audit_info(),
            customer_id: CustomerId::new(),
            amount: default_facility(),
            terms: Box::new(default_terms()),
            account_ids: CreditFacilityAccountIds::new(),
            disbursal_credit_account_id: CalaAccountId::new(),
            approval_process_id: ApprovalProcessId::new(),
        }]
    }

    fn hydrate_accruals_in_facility(credit_facility: &mut CreditFacility) {
        let new_entities = credit_facility
            .interest_accruals
            .new_entities_mut()
            .drain(..)
            .map(|new| {
                InterestAccrualCycle::try_from_events(new.into_events()).expect("hydrate failed")
            })
            .collect::<Vec<_>>();
        credit_facility
            .interest_accruals
            .extend_entities(new_entities);
    }

    #[test]
    fn collateral() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(credit_facility.collateral(), Satoshis::ZERO);

        credit_facility
            .record_collateral_update(
                Satoshis::from(10000),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();
        assert_eq!(credit_facility.collateral(), Satoshis::from(10000));

        credit_facility
            .record_collateral_update(
                Satoshis::from(5000),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();
        assert_eq!(credit_facility.collateral(), Satoshis::from(5000));
    }

    #[test]
    fn collateralization_ratio() {
        let events = initial_events();
        let mut credit_facility = facility_from(events);
        assert_eq!(
            credit_facility.collateralization_ratio(),
            Some(Decimal::ZERO)
        );

        credit_facility
            .record_collateral_update(
                Satoshis::from(5000),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();
        assert_eq!(credit_facility.collateralization_ratio(), Some(dec!(5)));
    }

    #[test]
    fn collateralization_ratio_when_active_disbursal() {
        let mut events = initial_events();
        let disbursal_amount = UsdCents::from(10);
        let disbursal_obligation_id = ObligationId::new();
        events.extend([
            CreditFacilityEvent::CollateralUpdated {
                tx_id: LedgerTxId::new(),
                total_collateral: Satoshis::from(500),
                abs_diff: Satoshis::from(500),
                action: CollateralAction::Add,
                recorded_in_ledger_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::BalanceUpdated {
                source: BalanceUpdatedSource::Obligation(disbursal_obligation_id),
                balance_type: BalanceUpdatedType::Disbursal,
                amount: disbursal_amount,
                updated_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);

        let credit_facility = facility_from(events);
        assert_eq!(credit_facility.collateralization_ratio(), Some(dec!(50)));
    }

    #[test]
    fn next_interest_accrual_cycle_period_handles_first_and_second_periods() {
        let mut events = initial_events();
        events.extend([CreditFacilityEvent::Activated {
            ledger_tx_id: LedgerTxId::new(),
            audit_info: dummy_audit_info(),
            activated_at: Utc::now(),
        }]);
        let mut credit_facility = facility_from(events);

        let first_accrual_cycle_period = credit_facility
            .next_interest_accrual_cycle_period()
            .unwrap()
            .unwrap();
        let InterestPeriod { start, .. } = first_accrual_cycle_period;
        assert_eq!(
            Utc::now().format("%Y-%m-%d").to_string(),
            start.format("%Y-%m-%d").to_string()
        );

        credit_facility
            .start_interest_accrual_cycle(dummy_audit_info())
            .unwrap()
            .unwrap();

        let second_accrual_period = credit_facility
            .next_interest_accrual_cycle_period()
            .unwrap()
            .unwrap();
        assert_eq!(first_accrual_cycle_period.next(), second_accrual_period);
    }

    #[test]
    fn next_interest_accrual_cycle_period_handles_last_period() {
        let mut events = initial_events();
        events.extend([CreditFacilityEvent::Activated {
            ledger_tx_id: LedgerTxId::new(),
            audit_info: dummy_audit_info(),
            activated_at: Utc::now(),
        }]);
        let mut credit_facility = facility_from(events);

        credit_facility
            .start_interest_accrual_cycle(dummy_audit_info())
            .unwrap()
            .unwrap();
        hydrate_accruals_in_facility(&mut credit_facility);

        let mut accrual_period = credit_facility
            .terms
            .accrual_cycle_interval
            .period_from(credit_facility.activated_at().expect("Not activated"));
        let mut next_accrual_period = credit_facility
            .next_interest_accrual_cycle_period()
            .unwrap();
        while next_accrual_period.is_some() {
            let new_idx = credit_facility
                .interest_accrual_cycle_in_progress()
                .expect("Interest accrual not found")
                .idx
                .next();
            let _ = credit_facility.record_interest_accrual_cycle(dummy_audit_info());

            let accrual_starts_at = next_accrual_period.unwrap().start;
            let id = InterestAccrualCycleId::new();
            credit_facility
                .events
                .push(CreditFacilityEvent::InterestAccrualCycleStarted {
                    interest_accrual_id: id,
                    idx: new_idx,
                    started_at: accrual_starts_at,
                    audit_info: dummy_audit_info(),
                });
            let new_accrual = NewInterestAccrualCycle::builder()
                .id(id)
                .credit_facility_id(credit_facility.id)
                .account_ids(credit_facility.account_ids.into())
                .idx(new_idx)
                .started_at(accrual_starts_at)
                .facility_matures_at(
                    credit_facility
                        .matures_at
                        .expect("Facility is already approved"),
                )
                .terms(credit_facility.terms)
                .audit_info(dummy_audit_info())
                .build()
                .unwrap();
            credit_facility.interest_accruals.add_new(new_accrual);
            hydrate_accruals_in_facility(&mut credit_facility);

            accrual_period = next_accrual_period.expect("Accrual period not found");
            next_accrual_period = credit_facility
                .next_interest_accrual_cycle_period()
                .unwrap();
        }
        assert_eq!(
            accrual_period.start.format("%Y-%m").to_string(),
            credit_facility
                .matures_at
                .unwrap()
                .format("%Y-%m")
                .to_string()
        );
    }

    #[test]
    fn cvl_check_approval_allowed() {
        let terms = default_terms();

        let facility_cvl = FacilityCVL {
            total: terms.margin_call_cvl - CVLPct::from(dec!(1)),
            disbursed: CVLPct::ZERO,
        };
        assert!(matches!(
            facility_cvl.check_approval_allowed(terms),
            Err(CreditFacilityError::BelowMarginLimit),
        ));

        let facility_cvl = FacilityCVL {
            total: terms.margin_call_cvl,
            disbursed: CVLPct::ZERO,
        };
        assert!(matches!(facility_cvl.check_approval_allowed(terms), Ok(())));
    }

    #[test]
    fn cvl_check_disbursal_allowed() {
        let terms = default_terms();

        let facility_cvl = FacilityCVL {
            total: terms.liquidation_cvl,
            disbursed: terms.margin_call_cvl - CVLPct::from(dec!(1)),
        };
        assert!(matches!(
            facility_cvl.check_disbursal_allowed(terms),
            Err(CreditFacilityError::BelowMarginLimit),
        ));

        let facility_cvl = FacilityCVL {
            total: terms.liquidation_cvl,
            disbursed: terms.margin_call_cvl,
        };
        assert!(matches!(
            facility_cvl.check_disbursal_allowed(terms),
            Ok(())
        ));
    }

    #[test]
    fn cvl_check_disbursal_allowed_for_zero_amount() {
        let terms = default_terms();

        let facility_cvl = FacilityCVL {
            total: terms.margin_call_cvl,
            disbursed: CVLPct::ZERO,
        };
        assert!(matches!(
            facility_cvl.check_disbursal_allowed(terms),
            Ok(())
        ));
    }

    #[test]
    fn check_activated_at() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(credit_facility.activated_at, None);
        assert_eq!(credit_facility.matures_at, None);

        credit_facility
            .record_collateral_update(
                default_full_collateral(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();
        let approval_time = Utc::now();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();

        assert!(credit_facility
            .activate(
                approval_time,
                default_price(),
                default_balances(credit_facility.amount),
                dummy_audit_info()
            )
            .unwrap()
            .did_execute());
        assert_eq!(credit_facility.activated_at, Some(approval_time));
        assert!(credit_facility.matures_at.is_some())
    }

    #[test]
    fn cannot_activate_if_credit_facility_has_no_collateral() {
        let mut events = initial_events();
        events.push({
            CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            }
        });
        let mut credit_facility = facility_from(events);
        let approval_time = Utc::now();
        let res = credit_facility.activate(
            approval_time,
            default_price(),
            default_balances(credit_facility.amount),
            dummy_audit_info(),
        );
        assert!(matches!(res, Err(CreditFacilityError::NoCollateral)));
    }

    #[test]
    fn reject_credit_facility_activate_below_margin_limit() {
        let mut credit_facility = facility_from(initial_events());

        credit_facility
            .record_collateral_update(
                Satoshis::from(100),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let approval_time = Utc::now();
        let res = credit_facility.activate(
            approval_time,
            default_price(),
            default_balances(credit_facility.amount),
            dummy_audit_info(),
        );
        assert!(matches!(res, Err(CreditFacilityError::BelowMarginLimit)));
    }

    #[test]
    fn status() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(
            credit_facility.status(),
            CreditFacilityStatus::PendingCollateralization
        );

        credit_facility
            .record_collateral_update(
                default_full_collateral(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount),
            )
            .unwrap();
        assert_eq!(
            credit_facility.status(),
            CreditFacilityStatus::PendingApproval
        );
        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        assert!(credit_facility
            .activate(
                Utc::now(),
                default_price(),
                default_balances(credit_facility.amount),
                dummy_audit_info()
            )
            .unwrap()
            .did_execute());
        assert_eq!(credit_facility.status(), CreditFacilityStatus::Active);
    }

    #[test]
    fn structuring_fee() {
        let credit_facility = facility_from(initial_events());
        let expected_fee = default_terms().one_time_fee_rate.apply(default_facility());
        assert_eq!(credit_facility.structuring_fee(), expected_fee);
    }

    mod activate {
        use super::*;

        #[test]
        fn errors_when_not_approved_yet() {
            let mut credit_facility = facility_from(initial_events());
            assert!(matches!(
                credit_facility.activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                ),
                Err(CreditFacilityError::ApprovalInProgress)
            ));
        }

        #[test]
        fn errors_if_denied() {
            let mut events = initial_events();
            events.push(CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: false,
                audit_info: dummy_audit_info(),
            });
            let mut credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                ),
                Err(CreditFacilityError::Denied)
            ));
        }

        #[test]
        fn errors_if_no_collateral() {
            let mut events = initial_events();
            events.push(CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            });
            let mut credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                ),
                Err(CreditFacilityError::NoCollateral)
            ));
        }

        #[test]
        fn errors_if_collateral_below_margin() {
            let mut events = initial_events();
            events.extend([
                CreditFacilityEvent::ApprovalProcessConcluded {
                    approval_process_id: ApprovalProcessId::new(),
                    approved: true,
                    audit_info: dummy_audit_info(),
                },
                CreditFacilityEvent::CollateralUpdated {
                    tx_id: LedgerTxId::new(),
                    total_collateral: Satoshis::ONE,
                    abs_diff: Satoshis::ONE,
                    action: CollateralAction::Add,
                    recorded_in_ledger_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
            ]);
            let mut credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                ),
                Err(CreditFacilityError::BelowMarginLimit)
            ));
        }

        #[test]
        fn errors_if_already_activated() {
            let mut events = initial_events();
            events.extend([
                CreditFacilityEvent::ApprovalProcessConcluded {
                    approval_process_id: ApprovalProcessId::new(),
                    approved: true,
                    audit_info: dummy_audit_info(),
                },
                CreditFacilityEvent::CollateralUpdated {
                    tx_id: LedgerTxId::new(),
                    total_collateral: Satoshis::ONE,
                    abs_diff: Satoshis::ONE,
                    action: CollateralAction::Add,
                    recorded_in_ledger_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
                CreditFacilityEvent::Activated {
                    ledger_tx_id: LedgerTxId::new(),
                    activated_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
            ]);
            let mut credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                ),
                Ok(Idempotent::Ignored)
            ));
        }

        #[test]
        fn can_activate() {
            let mut events = initial_events();
            let collateral_amount = Satoshis::from(1_000_000);
            events.extend([
                CreditFacilityEvent::ApprovalProcessConcluded {
                    approval_process_id: ApprovalProcessId::new(),
                    approved: true,
                    audit_info: dummy_audit_info(),
                },
                CreditFacilityEvent::CollateralUpdated {
                    tx_id: LedgerTxId::new(),
                    total_collateral: collateral_amount,
                    abs_diff: collateral_amount,
                    action: CollateralAction::Add,
                    recorded_in_ledger_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
            ]);
            let mut credit_facility = facility_from(events);

            assert!(credit_facility
                .activate(
                    Utc::now(),
                    default_price(),
                    default_balances(credit_facility.amount),
                    dummy_audit_info()
                )
                .is_ok());
        }
    }

    mod repayment {
        use super::*;

        impl From<CreditFacilityReceivable> for ObligationsAmounts {
            fn from(receivable: CreditFacilityReceivable) -> Self {
                Self {
                    disbursed: receivable.disbursed,
                    interest: receivable.interest,
                }
            }
        }

        // fn credit_facility_with_interest_accrual(
        //     facility_activated_at: DateTime<Utc>,
        // ) -> CreditFacility {
        //     let id = CreditFacilityId::new();
        //     let new_credit_facility = NewCreditFacility::builder()
        //         .id(id)
        //         .approval_process_id(id)
        //         .customer_id(CustomerId::new())
        //         .terms(default_terms())
        //         .facility(UsdCents::from(1_000_000_00))
        //         .account_ids(CreditFacilityAccountIds::new())
        //         .disbursal_credit_account_id(CalaAccountId::new())
        //         .audit_info(dummy_audit_info())
        //         .build()
        //         .expect("could not build new credit facility");
        //     let mut credit_facility =
        //         CreditFacility::try_from_events(new_credit_facility.into_events()).unwrap();

        //     credit_facility
        //         .record_collateral_update(
        //             Satoshis::from(50_00_000_000),
        //             dummy_audit_info(),
        //             default_price(),
        //             default_upgrade_buffer_cvl_pct(),
        //             ObligationsOutstanding::ZERO,
        //         )
        //         .unwrap();

        //     credit_facility
        //         .approval_process_concluded(true, dummy_audit_info())
        //         .unwrap();
        //     assert!(credit_facility
        //         .activate(facility_activated_at, default_price(), dummy_audit_info(),)
        //         .unwrap()
        //         .did_execute());
        //     hydrate_accruals_in_facility(&mut credit_facility);

        //     let new_disbursal = credit_facility
        //         .initiate_disbursal(
        //             UsdCents::from(600_000_00),
        //             ObligationsAmounts::ZERO,
        //             facility_activated_at,
        //             default_price(),
        //             None,
        //             dummy_audit_info(),
        //         )
        //         .unwrap();
        //     let mut disbursal = Disbursal::try_from_events(new_disbursal.into_events()).unwrap();
        //     let tx_id = LedgerTxId::new();
        //     let new_obligation = disbursal
        //         .approval_process_concluded(tx_id, true, dummy_audit_info())
        //         .unwrap();
        //     let obligation_id =
        //         new_obligation.map(|n| Obligation::try_from_events(n.into_events()).unwrap().id);
        //     credit_facility
        //         .disbursal_concluded(
        //             &disbursal,
        //             tx_id,
        //             obligation_id,
        //             facility_activated_at,
        //             dummy_audit_info(),
        //         )
        //         .unwrap();

        //     let mut accrual_cycle_data: Option<InterestAccrualCycleData> = None;
        //     while accrual_cycle_data.is_none() {
        //         let outstanding = credit_facility.total_outstanding();
        //         let accrual = credit_facility
        //             .interest_accrual_cycle_in_progress_mut()
        //             .unwrap();
        //         accrual.record_accrual(outstanding.into(), dummy_audit_info());
        //         accrual_cycle_data = accrual.accrual_cycle_data();
        //     }
        //     credit_facility
        //         .record_interest_accrual_cycle(dummy_audit_info())
        //         .unwrap();

        //     credit_facility
        // }

        // #[test]
        // fn confirm_completion() {
        //     let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        //     let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

        //     credit_facility
        //         .initiate_repayment(
        //             credit_facility.total_outstanding().total(),
        //             default_price(),
        //             default_upgrade_buffer_cvl_pct(),
        //             Utc::now(),
        //             dummy_audit_info(),
        //         )
        //         .unwrap();
        //     assert!(credit_facility.total_outstanding().is_zero());

        //     let _ = credit_facility
        //         .complete(
        //             dummy_audit_info(),
        //             default_price(),
        //             default_upgrade_buffer_cvl_pct(),
        //         )
        //         .unwrap();
        //     assert!(credit_facility.is_completed());
        //     assert!(credit_facility.status() == CreditFacilityStatus::Closed);
        // }
    }
}
