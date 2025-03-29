use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    primitives::*,
    terms::{CVLData, CVLPct, CollateralizationState, InterestPeriod, TermValues},
};

use crate::{disbursal::*, interest_accrual::*, ledger::*, payment::*};

use super::{error::CreditFacilityError, history, interest_outstanding, repayment_plan};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityId")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        terms: Box<TermValues>,
        facility: UsdCents,
        account_ids: CreditFacilityAccountIds,
        disbursal_credit_account_id: LedgerAccountId,
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
    DisbursalInitiated {
        disbursal_id: DisbursalId,
        idx: DisbursalIdx,
        approval_process_id: ApprovalProcessId,
        amount: UsdCents,
        audit_info: AuditInfo,
    },
    DisbursalConcluded {
        idx: DisbursalIdx,
        tx_id: Option<LedgerTxId>,
        canceled: bool,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    InterestAccrualStarted {
        interest_accrual_id: InterestAccrualId,
        idx: InterestAccrualIdx,
        started_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    InterestAccrualConcluded {
        idx: InterestAccrualIdx,
        tx_id: LedgerTxId,
        tx_ref: String,
        amount: UsdCents,
        accrued_at: DateTime<Utc>,
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
    PaymentRecorded {
        payment_id: PaymentId,
        disbursal_amount: UsdCents,
        interest_amount: UsdCents,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    OverdueDisbursedBalanceRecorded {
        amount: UsdCents,
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

impl From<CreditFacilityLedgerBalance> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityLedgerBalance) -> Self {
        Self {
            disbursed: balance.disbursed_receivable,
            interest: balance.interest_receivable,
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

    fn with_added_disbursal_amount(&self, amount: UsdCents) -> Self {
        Self {
            disbursed: self.disbursed + amount,
            interest: self.interest,
        }
    }

    fn facility_cvl_data(
        &self,
        collateral: Satoshis,
        facility_remaining: UsdCents,
    ) -> FacilityCVLData {
        FacilityCVLData {
            total: self.total_cvl(collateral, facility_remaining),
            disbursed: self.disbursed_cvl(collateral),
        }
    }

    fn allocate_payment(
        &self,
        amount: UsdCents,
    ) -> Result<CreditFacilityPaymentAmounts, CreditFacilityError> {
        if self.total() < amount {
            return Err(
                CreditFacilityError::PaymentExceedsOutstandingCreditFacilityAmount(
                    amount,
                    self.total(),
                ),
            );
        }

        let mut remaining = amount;

        let interest = std::cmp::min(amount, self.interest);
        remaining -= interest;

        let disbursal = std::cmp::min(remaining, self.disbursed);
        remaining -= disbursal;

        Ok(CreditFacilityPaymentAmounts {
            interest,
            disbursal,
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityBalance {
    pub facility_remaining: UsdCents,
    pub collateral: Satoshis,
    pub total_disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    pub due_disbursed_receivable: UsdCents,
    pub total_interest_accrued: UsdCents,
    pub interest_receivable: UsdCents,
    pub due_interest_receivable: UsdCents,
}

impl PartialEq<CreditFacilityLedgerBalance> for CreditFacilityBalance {
    fn eq(&self, other: &CreditFacilityLedgerBalance) -> bool {
        self.facility_remaining == other.facility
            && self.collateral == other.collateral
            && self.total_disbursed == other.disbursed
            && self.disbursed_receivable == other.disbursed_receivable
            && self.total_interest_accrued == other.interest
            && self.interest_receivable == other.interest_receivable
    }
}

impl CreditFacilityBalance {
    pub fn check_disbursal_amount(&self, amount: UsdCents) -> Result<(), CreditFacilityError> {
        if amount > self.facility_remaining {
            return Err(CreditFacilityError::DisbursalAmountTooLarge(
                amount,
                self.facility_remaining,
            ));
        }
        Ok(())
    }

    pub fn check_against_ledger(
        &self,
        ledger_balances: CreditFacilityLedgerBalance,
    ) -> Result<(), CreditFacilityError> {
        if *self != ledger_balances {
            return Err(CreditFacilityError::FacilityLedgerBalanceMismatch);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct FacilityCVLData {
    pub total: CVLData,
    pub disbursed: CVLData,
}

impl FacilityCVLData {
    pub fn cvl(&self, price: PriceOfOneBTC) -> FacilityCVL {
        FacilityCVL {
            total: self.total.cvl(price),
            disbursed: self.disbursed.cvl(price),
        }
    }
}

pub struct FacilityCVL {
    pub total: CVLPct,
    pub disbursed: CVLPct,
}

impl FacilityCVL {
    fn check_approval_allowed(&self, terms: TermValues) -> Result<(), CreditFacilityError> {
        if self.total < terms.margin_call_cvl {
            return Err(CreditFacilityError::BelowMarginLimit);
        }
        Ok(())
    }

    fn check_disbursal_allowed(&self, terms: TermValues) -> Result<(), CreditFacilityError> {
        let cvl = if self.disbursed.is_zero() {
            self.total
        } else {
            self.disbursed
        };

        if cvl < terms.margin_call_cvl {
            return Err(CreditFacilityError::BelowMarginLimit);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct NewAccrualPeriods {
    pub(crate) incurrence: InterestPeriod,
    pub(super) _accrual: InterestPeriod,
}

impl From<(InterestIncurrenceData, CreditFacilityAccountIds)> for CreditFacilityInterestIncurrence {
    fn from(data: (InterestIncurrenceData, CreditFacilityAccountIds)) -> Self {
        let (
            InterestIncurrenceData {
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

impl From<(InterestAccrualData, CreditFacilityAccountIds)> for CreditFacilityInterestAccrual {
    fn from(data: (InterestAccrualData, CreditFacilityAccountIds)) -> Self {
        let (
            InterestAccrualData {
                interest,
                tx_ref,
                tx_id,
                accrued_at,
            },
            credit_facility_account_ids,
        ) = data;
        Self {
            interest,
            tx_ref,
            tx_id,
            accrued_at,
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
    pub terms: TermValues,
    pub account_ids: CreditFacilityAccountIds,
    pub disbursal_credit_account_id: LedgerAccountId,
    #[builder(setter(strip_option), default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub matures_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub defaults_at: Option<DateTime<Utc>>,

    #[es_entity(nested)]
    #[builder(default)]
    interest_accruals: Nested<InterestAccrual>,
    events: EntityEvents<CreditFacilityEvent>,
}

impl CreditFacility {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn disbursed_overdue_at(&self) -> DateTime<Utc> {
        self.matures_at.expect("Facility not activated yet")
    }

    pub fn initial_facility(&self) -> UsdCents {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::Initialized { facility, .. } => return *facility,
                _ => continue,
            }
        }
        UsdCents::ZERO
    }

    pub fn activated_at(&self) -> Option<DateTime<Utc>> {
        self.events.iter_all().find_map(|event| match event {
            CreditFacilityEvent::Activated { activated_at, .. } => Some(*activated_at),
            _ => None,
        })
    }

    pub fn structuring_fee(&self) -> UsdCents {
        self.terms.one_time_fee_rate.apply(self.initial_facility())
    }

    fn total_disbursed(&self) -> UsdCents {
        let mut amounts = std::collections::HashMap::new();
        self.events
            .iter_all()
            .fold(UsdCents::from(0), |mut total_sum, event| {
                match event {
                    CreditFacilityEvent::DisbursalInitiated { idx, amount, .. } => {
                        amounts.insert(*idx, *amount);
                    }
                    CreditFacilityEvent::DisbursalConcluded {
                        idx,
                        tx_id: Some(_),
                        ..
                    } => {
                        if let Some(amount) = amounts.remove(idx) {
                            total_sum += amount;
                        }
                    }
                    _ => (),
                }
                total_sum
            })
    }

    fn disbursed_total_outstanding(&self) -> UsdCents {
        self.total_disbursed() - self.disbursed_payments()
    }

    fn _disbursed_outstanding_due(&self) -> UsdCents {
        if self.is_after_disbursed_default_date() || self.is_after_disbursed_overdue_date() {
            UsdCents::ZERO
        } else {
            self.disbursed_total_outstanding()
        }
    }

    fn disbursed_outstanding_overdue(&self) -> UsdCents {
        if self.is_after_disbursed_default_date() {
            UsdCents::ZERO
        } else if self.is_after_disbursed_overdue_date() {
            self.disbursed_total_outstanding()
        } else {
            UsdCents::ZERO
        }
    }

    fn _disbursed_outstanding_defaulted(&self) -> UsdCents {
        if self.is_after_disbursed_default_date() {
            self.disbursed_total_outstanding()
        } else {
            UsdCents::ZERO
        }
    }

    fn interest_total_outstanding(&self) -> UsdCents {
        interest_outstanding::project(self.events.iter_all()).total()
    }

    fn _interest_outstanding_due(&self) -> UsdCents {
        interest_outstanding::project(self.events.iter_all()).due
    }

    fn interest_outstanding_overdue(&self) -> UsdCents {
        interest_outstanding::project(self.events.iter_all()).overdue
    }

    fn _interest_outstanding_defaulted(&self) -> UsdCents {
        interest_outstanding::project(self.events.iter_all()).defaulted
    }

    fn facility_remaining(&self) -> UsdCents {
        self.initial_facility() - self.total_disbursed()
    }

    fn interest_accrued(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                CreditFacilityEvent::InterestAccrualConcluded { amount, .. } => Some(*amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    fn disbursed_payments(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                CreditFacilityEvent::PaymentRecorded {
                    disbursal_amount, ..
                } => Some(*disbursal_amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
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

    pub fn is_after_disbursed_overdue_date(&self) -> bool {
        self.is_after_maturity_date()
    }

    pub fn is_after_disbursed_default_date(&self) -> bool {
        let now = crate::time::now();
        self.defaults_at
            .is_some_and(|defaults_at| now > defaults_at)
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

        self.facility_cvl_data()
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
            .start_interest_accrual(audit_info)
            .expect("first accrual")
            .expect("first accrual");
        let activation = CreditFacilityActivation {
            tx_id,
            tx_ref: format!("{}-activate", self.id),
            credit_facility_account_ids: self.account_ids,
            debit_account_id: self.disbursal_credit_account_id,
            facility_amount: self.initial_facility(),
            structuring_fee_amount: self.structuring_fee(),
        };

        Ok(Idempotent::Executed((activation, periods.incurrence)))
    }

    pub(crate) fn initiate_disbursal(
        &mut self,
        amount: UsdCents,
        initiated_at: DateTime<Utc>,
        price: PriceOfOneBTC,
        approval_process_id: Option<ApprovalProcessId>,
        audit_info: AuditInfo,
    ) -> Idempotent<NewDisbursal> {
        if let Some(matures_at) = self.matures_at {
            if initiated_at > matures_at {
                return Idempotent::Ignored;
            }
        }

        if self
            .projected_cvl_data_for_disbursal(amount)
            .cvl(price)
            .check_disbursal_allowed(self.terms)
            .is_err()
        {
            return Idempotent::Ignored;
        };

        let idx = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::DisbursalInitiated { idx, .. } => Some(idx.next()),
                _ => None,
            })
            .unwrap_or(DisbursalIdx::FIRST);

        let disbursal_id = DisbursalId::new();
        let approval_process_id = approval_process_id.unwrap_or(disbursal_id.into());
        self.events.push(CreditFacilityEvent::DisbursalInitiated {
            disbursal_id,
            approval_process_id,
            idx,
            amount,
            audit_info: audit_info.clone(),
        });

        Idempotent::Executed(
            NewDisbursal::builder()
                .id(disbursal_id)
                .approval_process_id(approval_process_id)
                .credit_facility_id(self.id)
                .idx(idx)
                .amount(amount)
                .account_ids(self.account_ids)
                .disbursal_credit_account_id(self.disbursal_credit_account_id)
                .audit_info(audit_info)
                .build()
                .expect("could not build new disbursal"),
        )
    }

    pub(crate) fn disbursal_concluded(
        &mut self,
        disbursal: &Disbursal,
        tx_id: Option<LedgerTxId>,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events().iter_all(),
            CreditFacilityEvent::DisbursalConcluded {
                idx,
                ..
            } if idx == &disbursal.idx
        );

        self.events.push(CreditFacilityEvent::DisbursalConcluded {
            idx: disbursal.idx,
            recorded_at: executed_at,
            tx_id,
            canceled: tx_id.is_none(),
            audit_info,
        });
        Idempotent::Executed(())
    }

    fn next_interest_accrual_period(&self) -> Result<Option<InterestPeriod>, CreditFacilityError> {
        let last_accrual_start_date = self.events.iter_all().rev().find_map(|event| match event {
            CreditFacilityEvent::InterestAccrualStarted { started_at, .. } => Some(*started_at),
            _ => None,
        });

        let interval = self.terms.accrual_interval;
        let full_period = match last_accrual_start_date {
            Some(last_accrual_start_date) => interval.period_from(last_accrual_start_date).next(),
            None => interval.period_from(
                self.activated_at()
                    .ok_or(CreditFacilityError::NotActivatedYet)?,
            ),
        };

        Ok(full_period.truncate(self.matures_at.expect("Facility is already active")))
    }

    pub(crate) fn start_interest_accrual(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Option<NewAccrualPeriods>, CreditFacilityError> {
        let accrual_period = match self.next_interest_accrual_period()? {
            Some(period) => period,
            None => return Ok(None),
        };
        let now = crate::time::now();
        if accrual_period.start > now {
            return Err(CreditFacilityError::InterestAccrualWithInvalidFutureStartDate);
        }

        let idx = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualStarted { idx, .. } => Some(idx.next()),
                _ => None,
            })
            .unwrap_or(InterestAccrualIdx::FIRST);
        let id = InterestAccrualId::new();
        self.events
            .push(CreditFacilityEvent::InterestAccrualStarted {
                interest_accrual_id: id,
                idx,
                started_at: accrual_period.start,
                audit_info: audit_info.clone(),
            });

        let new_accrual = NewInterestAccrual::builder()
            .id(id)
            .credit_facility_id(self.id)
            .idx(idx)
            .started_at(accrual_period.start)
            .facility_matures_at(self.matures_at.expect("Facility is already approved"))
            .terms(self.terms)
            .audit_info(audit_info)
            .build()
            .expect("could not build new interest accrual");
        Ok(Some(NewAccrualPeriods {
            incurrence: self
                .interest_accruals
                .add_new(new_accrual)
                .first_incurrence_period(),
            _accrual: accrual_period,
        }))
    }

    pub(crate) fn record_interest_accrual(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<CreditFacilityInterestAccrual, CreditFacilityError> {
        let accrual_data = self
            .interest_accrual_in_progress()
            .expect("accrual not found")
            .accrual_data();
        let interest_accrual = accrual_data
            .map(|data| CreditFacilityInterestAccrual::from((data, self.account_ids)))
            .ok_or(CreditFacilityError::InterestAccrualNotCompletedYet)?;

        let idx = {
            let accrual = self
                .interest_accrual_in_progress()
                .expect("accrual not found");
            accrual.record_accrual(interest_accrual.clone(), audit_info.clone());
            accrual.idx
        };
        self.events
            .push(CreditFacilityEvent::InterestAccrualConcluded {
                idx,
                tx_id: interest_accrual.tx_id,
                tx_ref: interest_accrual.tx_ref.to_string(),
                amount: interest_accrual.interest,
                accrued_at: interest_accrual.accrued_at,
                audit_info,
            });

        Ok(interest_accrual)
    }

    pub fn interest_accrual_in_progress(&mut self) -> Option<&mut InterestAccrual> {
        if let Some(id) = self
            .events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualConcluded { .. } => Some(None),
                CreditFacilityEvent::InterestAccrualStarted {
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

    pub fn total_outstanding(&self) -> CreditFacilityReceivable {
        CreditFacilityReceivable {
            disbursed: self.disbursed_total_outstanding(),
            interest: self.interest_total_outstanding(),
        }
    }

    pub fn total_outstanding_after_added_disbursal_amount(
        &self,
        disbursal_amount: UsdCents,
    ) -> CreditFacilityReceivable {
        self.total_outstanding()
            .with_added_disbursal_amount(disbursal_amount)
    }

    pub fn outstanding_overdue(&self) -> CreditFacilityReceivable {
        CreditFacilityReceivable {
            disbursed: self.disbursed_outstanding_overdue(),
            interest: self.interest_outstanding_overdue(),
        }
    }

    pub fn balances(&self) -> CreditFacilityBalance {
        CreditFacilityBalance {
            facility_remaining: self.facility_remaining(),
            collateral: self.collateral(),
            total_disbursed: self.total_disbursed(),
            disbursed_receivable: self.total_outstanding().disbursed,
            due_disbursed_receivable: self.outstanding_overdue().disbursed,
            total_interest_accrued: self.interest_accrued(),
            interest_receivable: self.total_outstanding().interest,
            due_interest_receivable: self.outstanding_overdue().interest,
        }
    }

    pub fn can_be_completed(&self) -> bool {
        self.total_outstanding().is_zero()
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

    pub fn facility_cvl_data(&self) -> FacilityCVLData {
        self.total_outstanding()
            .facility_cvl_data(self.collateral(), self.facility_remaining())
    }

    fn payment_account_ids(&self) -> PaymentAccountIds {
        if self.has_overdue_disbursed_balance_recorded() {
            PaymentAccountIds {
                disbursed_receivable_account_id: self
                    .account_ids
                    .disbursed_receivable_overdue_account_id,
                interest_receivable_account_id: self.account_ids.interest_receivable_account_id,
            }
        } else {
            PaymentAccountIds {
                disbursed_receivable_account_id: self.account_ids.disbursed_receivable_account_id,
                interest_receivable_account_id: self.account_ids.interest_receivable_account_id,
            }
        }
    }

    pub(crate) fn initiate_repayment(
        &mut self,
        amount: UsdCents,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        now: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Result<NewPayment, CreditFacilityError> {
        if self.total_outstanding().is_zero() {
            return Err(
                CreditFacilityError::PaymentExceedsOutstandingCreditFacilityAmount(
                    self.total_outstanding().total(),
                    amount,
                ),
            );
        }

        let amounts = self.outstanding_overdue().allocate_payment(amount)?;

        let payment_id = PaymentId::new();
        let tx_ref = format!("{}-payment-{}", self.id, self.count_recorded_payments() + 1);

        self.events.push(CreditFacilityEvent::PaymentRecorded {
            payment_id,
            disbursal_amount: amounts.disbursal,
            interest_amount: amounts.interest,
            recorded_at: now,
            audit_info: audit_info.clone(),
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, &audit_info);

        Ok(NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .ledger_tx_ref(tx_ref)
            .credit_facility_id(self.id)
            .amounts(amounts)
            .account_ids(self.payment_account_ids())
            .disbursal_credit_account_id(self.disbursal_credit_account_id)
            .audit_info(audit_info)
            .build()
            .expect("could not build new payment"))
    }

    fn count_recorded_payments(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, CreditFacilityEvent::PaymentRecorded { .. }))
            .count()
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
        audit_info: &AuditInfo,
    ) -> Option<CollateralizationState> {
        let facility_cvl = self.facility_cvl_data().cvl(price);
        let last_collateralization_state = self.last_collateralization_state();

        let collateralization_update =
            match self.status() {
                CreditFacilityStatus::PendingCollateralization
                | CreditFacilityStatus::PendingApproval => facility_cvl
                    .total
                    .collateralization_update(self.terms, last_collateralization_state, None, true),
                CreditFacilityStatus::Active | CreditFacilityStatus::Matured => {
                    let cvl = if self.total_disbursed() == UsdCents::ZERO {
                        facility_cvl.total
                    } else {
                        facility_cvl.disbursed
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
                    outstanding: self.total_outstanding(),
                    price,
                    recorded_at: now,
                    audit_info: audit_info.clone(),
                });

            return Some(calculated_collateralization);
        }

        None
    }

    fn projected_cvl_data_for_disbursal(&self, disbursal_amount: UsdCents) -> FacilityCVLData {
        self.total_outstanding_after_added_disbursal_amount(disbursal_amount)
            .facility_cvl_data(self.collateral(), self.facility_remaining())
    }

    pub(crate) fn record_collateral_update(
        &mut self,
        updated_collateral: Satoshis,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
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

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, &audit_info);
    }

    pub(crate) fn has_overdue_disbursed_balance_recorded(&self) -> bool {
        self.events.iter_all().rev().any(|event| {
            matches!(
                event,
                CreditFacilityEvent::OverdueDisbursedBalanceRecorded { .. }
            )
        })
    }

    pub(crate) fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .rev()
            .any(|event| matches!(event, CreditFacilityEvent::Completed { .. }))
    }

    pub(crate) fn record_overdue_disbursed_balance(
        &mut self,
        audit_info: AuditInfo,
    ) -> Idempotent<CreditFacilityOverdueDisbursedBalance> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            CreditFacilityEvent::OverdueDisbursedBalanceRecorded { .. }
        );
        if self.is_completed() || self.total_outstanding().is_zero() {
            return Idempotent::Ignored;
        }

        let res = CreditFacilityOverdueDisbursedBalance {
            tx_id: LedgerTxId::new(),
            disbursed_outstanding: self.disbursed_outstanding_overdue(),
            credit_facility_account_ids: self.account_ids,
        };

        self.events
            .push(CreditFacilityEvent::OverdueDisbursedBalanceRecorded {
                amount: res.disbursed_outstanding,
                recorded_at: self.disbursed_overdue_at(),
                audit_info,
            });

        Idempotent::Executed(res)
    }

    pub(crate) fn complete(
        &mut self,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Result<Idempotent<CreditFacilityCompletion>, CreditFacilityError> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityEvent::Completed { .. }
        );
        if !self.total_outstanding().is_zero() {
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
        );

        self.events.push(CreditFacilityEvent::Completed {
            completed_at,
            audit_info,
        });

        Ok(Idempotent::Executed(res))
    }

    pub(super) fn collateralization_ratio(&self) -> Option<Decimal> {
        let amount = if self.status() == CreditFacilityStatus::PendingCollateralization
            || self.status() == CreditFacilityStatus::PendingApproval
            || self.total_disbursed() == UsdCents::ZERO
        {
            self.initial_facility()
        } else {
            self.total_outstanding().total()
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

    pub(crate) fn disbursal_amount_from_idx(&self, idx: DisbursalIdx) -> UsdCents {
        if let Some(amount) = self
            .events
            .iter_all()
            .filter_map(|event| match event {
                CreditFacilityEvent::DisbursalInitiated { idx: i, amount, .. } if i == &idx => {
                    Some(amount)
                }
                _ => None,
            })
            .next()
        {
            if self.events.iter_all().any(|event| {
                matches!(
                    event,
                    CreditFacilityEvent::DisbursalConcluded { idx: i, tx_id: Some(_), .. } if i == &idx
                )
            }) {
                return *amount;
            }
        }
        UsdCents::ZERO
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
                CreditFacilityEvent::DisbursalInitiated { .. } => (),
                CreditFacilityEvent::DisbursalConcluded { .. } => (),
                CreditFacilityEvent::InterestAccrualStarted { .. } => (),
                CreditFacilityEvent::InterestAccrualConcluded { .. } => (),
                CreditFacilityEvent::CollateralUpdated { .. } => (),
                CreditFacilityEvent::CollateralizationChanged { .. } => (),
                CreditFacilityEvent::PaymentRecorded { .. } => (),
                CreditFacilityEvent::OverdueDisbursedBalanceRecorded { .. } => (),
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
    facility: UsdCents,
    #[builder(setter(skip), default)]
    pub(super) status: CreditFacilityStatus,
    #[builder(setter(skip), default)]
    pub(super) collateralization_state: CollateralizationState,
    account_ids: CreditFacilityAccountIds,
    disbursal_credit_account_id: LedgerAccountId,
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
                facility: self.facility,
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
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
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

    fn facility_from(events: Vec<CreditFacilityEvent>) -> CreditFacility {
        CreditFacility::try_from_events(EntityEvents::init(CreditFacilityId::new(), events))
            .unwrap()
    }

    fn initial_events() -> Vec<CreditFacilityEvent> {
        vec![CreditFacilityEvent::Initialized {
            id: CreditFacilityId::new(),
            audit_info: dummy_audit_info(),
            customer_id: CustomerId::new(),
            facility: default_facility(),
            terms: Box::new(default_terms()),
            account_ids: CreditFacilityAccountIds::new(),
            disbursal_credit_account_id: LedgerAccountId::new(),
            approval_process_id: ApprovalProcessId::new(),
        }]
    }

    fn hydrate_accruals_in_facility(credit_facility: &mut CreditFacility) {
        let new_entities = credit_facility
            .interest_accruals
            .new_entities_mut()
            .drain(..)
            .map(|new| InterestAccrual::try_from_events(new.into_events()).expect("hydrate failed"))
            .collect::<Vec<_>>();
        credit_facility
            .interest_accruals
            .extend_entities(new_entities);
    }

    #[test]
    fn allocate_payment() {
        let outstanding = CreditFacilityReceivable {
            disbursed: UsdCents::from(100),
            interest: UsdCents::from(2),
        };
        assert!(outstanding.allocate_payment(UsdCents::from(200)).is_err());
        assert!(outstanding.allocate_payment(UsdCents::from(100)).is_ok());
    }

    #[test]
    fn is_disbursal_in_progress() {
        let mut events = initial_events();

        let first_idx = DisbursalIdx::FIRST;
        let disbursal_id = DisbursalId::new();
        events.extend([
            CreditFacilityEvent::CollateralUpdated {
                tx_id: LedgerTxId::new(),
                total_collateral: Satoshis::from(500),
                abs_diff: Satoshis::from(500),
                action: CollateralAction::Add,
                recorded_in_ledger_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id,
                approval_process_id: disbursal_id.into(),
                idx: first_idx,
                amount: UsdCents::ONE,
                audit_info: dummy_audit_info(),
            },
        ]);

        events.push(CreditFacilityEvent::DisbursalConcluded {
            idx: first_idx,
            tx_id: Some(LedgerTxId::new()),
            canceled: false,
            recorded_at: Utc::now(),
            audit_info: dummy_audit_info(),
        });
        assert!(facility_from(events)
            .initiate_disbursal(
                UsdCents::ONE,
                Utc::now(),
                default_price(),
                None,
                dummy_audit_info()
            )
            .did_execute());
    }

    #[test]
    fn interest_accrued() {
        let mut events = initial_events();
        events.extend([
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: InterestAccrualIdx::FIRST,
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(10),
                accrued_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::InterestAccrualConcluded {
                idx: InterestAccrualIdx::FIRST.next(),
                tx_id: LedgerTxId::new(),
                tx_ref: "".to_string(),
                amount: UsdCents::from(20),
                accrued_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(credit_facility.interest_accrued(), UsdCents::from(30));
    }

    #[test]
    fn outstanding() {
        let mut events = initial_events();
        let disbursal_id = DisbursalId::new();
        events.extend([
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id,
                approval_process_id: disbursal_id.into(),
                idx: DisbursalIdx::FIRST,
                amount: UsdCents::from(100),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: DisbursalIdx::FIRST,
                tx_id: Some(LedgerTxId::new()),
                canceled: false,
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.total_outstanding(),
            CreditFacilityReceivable {
                disbursed: UsdCents::from(100),
                interest: UsdCents::ZERO
            }
        );
    }

    #[test]
    fn outstanding_overdue_before_maturity() {
        let mut events = initial_events();
        let activated_at = Utc::now();
        let disbursal_id = DisbursalId::new();
        events.extend([
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id,
                approval_process_id: disbursal_id.into(),
                idx: DisbursalIdx::FIRST,
                amount: UsdCents::from(100),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: DisbursalIdx::FIRST,
                tx_id: Some(LedgerTxId::new()),
                canceled: false,
                recorded_at: activated_at,
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.outstanding_overdue().disbursed,
            UsdCents::ZERO
        );
    }

    #[test]
    fn outstanding_overdue_after_maturity() {
        let mut events = initial_events();
        let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let disbursal_id = DisbursalId::new();
        events.extend([
            CreditFacilityEvent::Activated {
                ledger_tx_id: LedgerTxId::new(),
                activated_at,
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id,
                approval_process_id: disbursal_id.into(),
                idx: DisbursalIdx::FIRST,
                amount: UsdCents::from(100),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: DisbursalIdx::FIRST,
                tx_id: Some(LedgerTxId::new()),
                canceled: false,
                recorded_at: activated_at,
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.outstanding_overdue().disbursed,
            UsdCents::from(100)
        );
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
            )
            .unwrap();
        assert_eq!(credit_facility.collateral(), Satoshis::from(10000));

        credit_facility
            .record_collateral_update(
                Satoshis::from(5000),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
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
            )
            .unwrap();
        assert_eq!(credit_facility.collateralization_ratio(), Some(dec!(5)));
    }

    #[test]
    fn collateralization_ratio_when_active_disbursal() {
        let mut events = initial_events();
        let disbursal_id = DisbursalId::new();
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
            CreditFacilityEvent::DisbursalInitiated {
                disbursal_id,
                approval_process_id: disbursal_id.into(),
                idx: DisbursalIdx::FIRST,
                amount: UsdCents::from(10),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursalConcluded {
                idx: DisbursalIdx::FIRST,
                tx_id: Some(LedgerTxId::new()),
                canceled: false,
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);

        let credit_facility = facility_from(events);
        assert_eq!(credit_facility.collateralization_ratio(), Some(dec!(50)));
    }

    #[test]
    fn next_interest_accrual_period_handles_first_and_second_periods() {
        let mut events = initial_events();
        events.extend([CreditFacilityEvent::Activated {
            ledger_tx_id: LedgerTxId::new(),
            audit_info: dummy_audit_info(),
            activated_at: Utc::now(),
        }]);
        let mut credit_facility = facility_from(events);

        let first_accrual_period = credit_facility
            .next_interest_accrual_period()
            .unwrap()
            .unwrap();
        let InterestPeriod { start, .. } = first_accrual_period;
        assert_eq!(
            Utc::now().format("%Y-%m-%d").to_string(),
            start.format("%Y-%m-%d").to_string()
        );

        credit_facility
            .start_interest_accrual(dummy_audit_info())
            .unwrap()
            .unwrap();

        let second_accrual_period = credit_facility
            .next_interest_accrual_period()
            .unwrap()
            .unwrap();
        assert_eq!(first_accrual_period.next(), second_accrual_period);
    }

    #[test]
    fn next_interest_accrual_period_handles_last_period() {
        let mut events = initial_events();
        events.extend([CreditFacilityEvent::Activated {
            ledger_tx_id: LedgerTxId::new(),
            audit_info: dummy_audit_info(),
            activated_at: Utc::now(),
        }]);
        let mut credit_facility = facility_from(events);

        credit_facility
            .start_interest_accrual(dummy_audit_info())
            .unwrap()
            .unwrap();
        hydrate_accruals_in_facility(&mut credit_facility);

        let mut accrual_period = credit_facility
            .terms
            .accrual_interval
            .period_from(credit_facility.activated_at().expect("Not activated"));
        let mut next_accrual_period = credit_facility.next_interest_accrual_period().unwrap();
        while next_accrual_period.is_some() {
            let new_idx = credit_facility
                .interest_accrual_in_progress()
                .expect("Interest accrual not found")
                .idx
                .next();
            let _ = credit_facility.record_interest_accrual(dummy_audit_info());

            let accrual_starts_at = next_accrual_period.unwrap().start;
            let id = InterestAccrualId::new();
            credit_facility
                .events
                .push(CreditFacilityEvent::InterestAccrualStarted {
                    interest_accrual_id: id,
                    idx: new_idx,
                    started_at: accrual_starts_at,
                    audit_info: dummy_audit_info(),
                });
            let new_accrual = NewInterestAccrual::builder()
                .id(id)
                .credit_facility_id(credit_facility.id)
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
            next_accrual_period = credit_facility.next_interest_accrual_period().unwrap();
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
            )
            .unwrap();
        let approval_time = Utc::now();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();

        assert!(credit_facility
            .activate(approval_time, default_price(), dummy_audit_info())
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
        let res = credit_facility.activate(approval_time, default_price(), dummy_audit_info());
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
            )
            .unwrap();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let approval_time = Utc::now();
        let res = credit_facility.activate(approval_time, default_price(), dummy_audit_info());
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
            .activate(Utc::now(), default_price(), dummy_audit_info())
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
                credit_facility.activate(Utc::now(), default_price(), dummy_audit_info()),
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
                credit_facility.activate(Utc::now(), default_price(), dummy_audit_info()),
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
                credit_facility.activate(Utc::now(), default_price(), dummy_audit_info()),
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
                credit_facility.activate(Utc::now(), default_price(), dummy_audit_info()),
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
                credit_facility.activate(Utc::now(), default_price(), dummy_audit_info()),
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
                .activate(Utc::now(), default_price(), dummy_audit_info())
                .is_ok());
        }
    }

    mod repayment {
        use super::*;

        fn credit_facility_with_interest_accrual(
            facility_activated_at: DateTime<Utc>,
        ) -> CreditFacility {
            let id = CreditFacilityId::new();
            let new_credit_facility = NewCreditFacility::builder()
                .id(id)
                .approval_process_id(id)
                .customer_id(CustomerId::new())
                .terms(default_terms())
                .facility(UsdCents::from(1_000_000_00))
                .account_ids(CreditFacilityAccountIds::new())
                .disbursal_credit_account_id(LedgerAccountId::new())
                .audit_info(dummy_audit_info())
                .build()
                .expect("could not build new credit facility");
            let mut credit_facility =
                CreditFacility::try_from_events(new_credit_facility.into_events()).unwrap();

            credit_facility
                .record_collateral_update(
                    Satoshis::from(50_00_000_000),
                    dummy_audit_info(),
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                )
                .unwrap();

            credit_facility
                .approval_process_concluded(true, dummy_audit_info())
                .unwrap();
            assert!(credit_facility
                .activate(facility_activated_at, default_price(), dummy_audit_info(),)
                .unwrap()
                .did_execute());
            hydrate_accruals_in_facility(&mut credit_facility);

            let new_disbursal = credit_facility
                .initiate_disbursal(
                    UsdCents::from(600_000_00),
                    facility_activated_at,
                    default_price(),
                    None,
                    dummy_audit_info(),
                )
                .unwrap();
            let mut disbursal = Disbursal::try_from_events(new_disbursal.into_events()).unwrap();
            let data = disbursal
                .approval_process_concluded(true, dummy_audit_info())
                .unwrap();
            credit_facility
                .disbursal_concluded(
                    &disbursal,
                    Some(data.tx_id),
                    facility_activated_at,
                    dummy_audit_info(),
                )
                .unwrap();

            let mut accrual_data: Option<InterestAccrualData> = None;
            while accrual_data.is_none() {
                let outstanding = credit_facility.total_outstanding();
                let accrual = credit_facility.interest_accrual_in_progress().unwrap();
                accrual.record_incurrence(outstanding, dummy_audit_info());
                accrual_data = accrual.accrual_data();
            }
            credit_facility
                .record_interest_accrual(dummy_audit_info())
                .unwrap();

            credit_facility
        }

        #[test]
        fn initiate_repayment_errors_when_no_disbursals() {
            let mut credit_facility = facility_from(initial_events());

            let repayment_amount = UsdCents::from(5);
            assert!(credit_facility
                .initiate_repayment(
                    repayment_amount,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .is_err());
        }

        #[test]
        fn initiate_repayment_before_maturity_errors_for_amount_above_interest() {
            let activated_at = Utc::now() - chrono::Duration::days(40);
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);
            let interest = credit_facility.total_outstanding().interest;

            assert!(credit_facility
                .initiate_repayment(
                    interest + UsdCents::ONE,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .is_err());
            assert!(credit_facility
                .initiate_repayment(
                    interest,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .is_ok());
            assert!(!credit_facility.is_after_maturity_date())
        }

        #[test]
        fn initiate_repayment_after_maturity_errors_for_amount_above_total() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);
            let outstanding = credit_facility.total_outstanding().total();

            assert!(credit_facility
                .initiate_repayment(
                    outstanding + UsdCents::ONE,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .is_err());
            assert!(credit_facility
                .initiate_repayment(
                    outstanding,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .is_ok());
            assert!(credit_facility.is_after_maturity_date())
        }

        #[test]
        fn initiate_repayment_after_overdue_event_returns_overdue_account() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let new_payment = credit_facility
                .initiate_repayment(
                    UsdCents::ONE,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .unwrap();
            let payment = Payment::try_from_events(new_payment.into_events()).unwrap();
            assert_eq!(
                payment.account_ids.disbursed_receivable_account_id,
                credit_facility.account_ids.disbursed_receivable_account_id,
            );

            let _ = credit_facility.record_overdue_disbursed_balance(dummy_audit_info());

            let new_payment = credit_facility
                .initiate_repayment(
                    UsdCents::ONE,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .unwrap();
            let payment = Payment::try_from_events(new_payment.into_events()).unwrap();
            assert_eq!(
                payment.account_ids.disbursed_receivable_account_id,
                credit_facility
                    .account_ids
                    .disbursed_receivable_overdue_account_id,
            );
        }

        #[test]
        fn confirm_repayment_before_maturity() {
            let activated_at = Utc::now() - chrono::Duration::days(40);
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let repayment_amount = credit_facility.total_outstanding().interest;
            let outstanding_before = credit_facility.total_outstanding();
            credit_facility
                .initiate_repayment(
                    repayment_amount,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .unwrap();

            let outstanding_after = credit_facility.total_outstanding();

            assert_eq!(
                outstanding_before.total() - outstanding_after.total(),
                repayment_amount
            );
            assert!(!credit_facility.is_after_maturity_date())
        }

        #[test]
        fn confirm_partial_repayment_after_maturity() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let partial_repayment_amount = credit_facility.total_outstanding().interest
                + credit_facility.total_outstanding().disbursed
                - UsdCents::from(100);
            let outstanding_before = credit_facility.total_outstanding();
            credit_facility
                .initiate_repayment(
                    partial_repayment_amount,
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .unwrap();
            let outstanding_after = credit_facility.total_outstanding();

            assert!(!outstanding_after.is_zero());
            assert_eq!(
                outstanding_before.total() - outstanding_after.total(),
                partial_repayment_amount
            );
            assert!(credit_facility.is_after_maturity_date())
        }

        #[test]
        fn confirm_completion() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            credit_facility
                .initiate_repayment(
                    credit_facility.total_outstanding().total(),
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    Utc::now(),
                    dummy_audit_info(),
                )
                .unwrap();
            assert!(credit_facility.total_outstanding().is_zero());

            let _ = credit_facility
                .complete(
                    dummy_audit_info(),
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                )
                .unwrap();
            assert!(credit_facility.is_completed());
            assert!(credit_facility.status() == CreditFacilityStatus::Closed);
        }
    }
}
