use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{
    audit::AuditInfo,
    ledger::{credit_facility::*, customer::CustomerLedgerAccountIds},
    primitives::*,
    terms::{CVLData, CVLPct, CollateralizationState, InterestPeriod, TermValues},
};

use super::{
    disbursal::*, history, interest_accrual::*, CreditFacilityCollateralUpdate, CreditFacilityError,
};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityId")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        terms: TermValues,
        facility: UsdCents,
        account_ids: CreditFacilityAccountIds,
        customer_account_ids: CustomerLedgerAccountIds,
        audit_info: AuditInfo,
    },
    ApprovalProcessStarted {
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
        tx_ref: String,
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
        tx_id: LedgerTxId,
        tx_ref: String,
        disbursal_amount: UsdCents,
        interest_amount: UsdCents,
        audit_info: AuditInfo,
        recorded_in_ledger_at: DateTime<Utc>,
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

impl From<CreditFacilityBalance> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityBalance) -> Self {
        Self {
            disbursed: balance.disbursed_receivable,
            interest: balance.accrued_interest_receivable,
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
    fn is_approval_allowed(&self, terms: TermValues) -> Result<(), CreditFacilityError> {
        if self.total < terms.margin_call_cvl {
            return Err(CreditFacilityError::BelowMarginLimit);
        }
        Ok(())
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
    pub customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(strip_option), default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub expires_at: Option<DateTime<Utc>>,

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

    fn disbursed_due(&self) -> UsdCents {
        if self.is_expired() {
            self.total_disbursed()
        } else {
            UsdCents::ZERO
        }
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

    fn interest_payments(&self) -> UsdCents {
        self.events
            .iter_all()
            .filter_map(|event| match event {
                CreditFacilityEvent::PaymentRecorded {
                    interest_amount, ..
                } => Some(*interest_amount),
                _ => None,
            })
            .fold(UsdCents::ZERO, |acc, amount| acc + amount)
    }

    pub fn history(&self) -> Vec<history::CreditFacilityHistoryEntry> {
        history::project(self.events.iter_all())
    }

    pub(super) fn is_approval_process_concluded(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::ApprovalProcessConcluded { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub(super) fn is_approved(&self) -> Result<bool, CreditFacilityError> {
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

    pub(super) fn is_activated(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                CreditFacilityEvent::Activated { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub(super) fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |expires_at| Utc::now() > expires_at)
    }

    pub fn status(&self) -> CreditFacilityStatus {
        if self.is_completed() {
            CreditFacilityStatus::Closed
        } else if self.is_expired() {
            CreditFacilityStatus::Expired
        } else if self.is_activated() {
            CreditFacilityStatus::Active
        } else if self.is_fully_collateralized() {
            CreditFacilityStatus::PendingApproval
        } else {
            CreditFacilityStatus::PendingCollateralization
        }
    }

    pub(super) fn approval_process_concluded(
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

    pub(crate) fn activation_data(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<CreditFacilityActivationData, CreditFacilityError> {
        if self.is_activated() {
            return Err(CreditFacilityError::AlreadyActivated);
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
            .is_approval_allowed(self.terms)?;

        Ok(CreditFacilityActivationData {
            facility: self.initial_facility(),
            tx_ref: format!("{}-activate", self.id),
            tx_id: LedgerTxId::new(),
            credit_facility_account_ids: self.account_ids,
            customer_account_ids: self.customer_account_ids,
        })
    }

    pub(super) fn activate(
        &mut self,
        CreditFacilityActivationData { tx_id, .. }: CreditFacilityActivationData,
        activated_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Idempotent<InterestPeriod> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityEvent::Activated { .. }
        );

        self.activated_at = Some(activated_at);
        self.expires_at = Some(self.terms.duration.expiration_date(activated_at));
        self.events.push(CreditFacilityEvent::Activated {
            ledger_tx_id: tx_id,
            activated_at,
            audit_info: audit_info.clone(),
        });

        let new_accrual = self
            .start_interest_accrual(audit_info)
            .expect("first accrual")
            .expect("first accrual");

        Idempotent::Executed(new_accrual.first_incurrence_period())
    }

    pub(super) fn initiate_disbursal(
        &mut self,
        amount: UsdCents,
        initiated_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) -> Result<NewDisbursal, CreditFacilityError> {
        if let Some(expires_at) = self.expires_at {
            if initiated_at > expires_at {
                return Err(CreditFacilityError::DisbursalPastExpiryDate);
            }
        }

        if self.is_disbursal_in_progress() {
            return Err(CreditFacilityError::DisbursalInProgress);
        }

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
        self.events.push(CreditFacilityEvent::DisbursalInitiated {
            disbursal_id,
            approval_process_id: disbursal_id.into(),
            idx,
            amount,
            audit_info: audit_info.clone(),
        });

        Ok(NewDisbursal::builder()
            .id(disbursal_id)
            .approval_process_id(disbursal_id)
            .credit_facility_id(self.id)
            .idx(idx)
            .amount(amount)
            .account_ids(self.account_ids)
            .customer_account_ids(self.customer_account_ids)
            .audit_info(audit_info)
            .build()
            .expect("could not build new disbursal"))
    }

    pub(super) fn confirm_disbursal(
        &mut self,
        disbursal: &Disbursal,
        tx_id: Option<LedgerTxId>,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events.push(CreditFacilityEvent::DisbursalConcluded {
            idx: disbursal.idx,
            recorded_at: executed_at,
            tx_id,
            audit_info,
        });
    }

    fn is_disbursal_in_progress(&self) -> bool {
        for event in self.events.iter_all().rev() {
            if let CreditFacilityEvent::DisbursalInitiated { .. } = event {
                return true;
            }
            if let CreditFacilityEvent::DisbursalConcluded { .. } = event {
                return false;
            }
        }

        false
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

        Ok(full_period.truncate(self.expires_at.expect("Facility is already active")))
    }

    pub(super) fn start_interest_accrual(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<Option<&NewInterestAccrual>, CreditFacilityError> {
        let accrual_starts_at = match self.next_interest_accrual_period()? {
            Some(period) => period,
            None => return Ok(None),
        }
        .start;
        if accrual_starts_at > Utc::now() {
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
                started_at: accrual_starts_at,
                audit_info: audit_info.clone(),
            });

        let new_accrual = NewInterestAccrual::builder()
            .id(id)
            .credit_facility_id(self.id)
            .idx(idx)
            .started_at(accrual_starts_at)
            .facility_expires_at(self.expires_at.expect("Facility is already approved"))
            .terms(self.terms)
            .audit_info(audit_info)
            .build()
            .expect("could not build new interest accrual");
        Ok(Some(self.interest_accruals.add_new(new_accrual)))
    }

    pub fn confirm_interest_accrual(
        &mut self,
        CreditFacilityInterestAccrual {
            interest,
            tx_ref,
            tx_id,
            accrued_at,
            ..
        }: CreditFacilityInterestAccrual,
        idx: InterestAccrualIdx,
        audit_info: AuditInfo,
    ) {
        self.events
            .push(CreditFacilityEvent::InterestAccrualConcluded {
                idx,
                tx_id,
                tx_ref,
                amount: interest,
                accrued_at,
                audit_info,
            });
    }

    pub fn interest_accrual_in_progress(&self) -> Option<InterestAccrualId> {
        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::InterestAccrualConcluded { .. } => Some(None),
                CreditFacilityEvent::InterestAccrualStarted {
                    interest_accrual_id: id,
                    ..
                } => Some(Some(*id)),
                _ => None,
            })
            .and_then(|id| id)
    }

    pub fn outstanding(&self) -> CreditFacilityReceivable {
        CreditFacilityReceivable {
            disbursed: self.total_disbursed() - self.disbursed_payments(),
            interest: self.interest_accrued() - self.interest_payments(),
        }
    }

    pub fn outstanding_from_due(&self) -> CreditFacilityReceivable {
        CreditFacilityReceivable {
            disbursed: std::cmp::max(
                self.disbursed_due() - self.disbursed_payments(),
                UsdCents::ZERO,
            ),
            interest: self.interest_accrued() - self.interest_payments(),
        }
    }

    pub fn can_be_completed(&self) -> bool {
        self.outstanding().is_zero()
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
        let total = self
            .outstanding()
            .total_cvl(self.collateral(), self.facility_remaining());
        let disbursed = self.outstanding().disbursed_cvl(self.collateral());

        FacilityCVLData { total, disbursed }
    }

    pub(super) fn initiate_repayment(
        &self,
        amount: UsdCents,
    ) -> Result<CreditFacilityRepayment, CreditFacilityError> {
        if self.outstanding().is_zero() {
            return Err(
                CreditFacilityError::PaymentExceedsOutstandingCreditFacilityAmount(
                    self.outstanding().total(),
                    amount,
                ),
            );
        }

        let amounts = self.outstanding_from_due().allocate_payment(amount)?;

        let tx_ref = format!("{}-payment-{}", self.id, self.count_recorded_payments() + 1);

        let res = CreditFacilityRepayment {
            tx_id: LedgerTxId::new(),
            tx_ref,
            credit_facility_account_ids: self.account_ids,
            customer_account_ids: self.customer_account_ids,
            amounts,
        };

        Ok(res)
    }

    pub fn confirm_repayment(
        &mut self,
        repayment: CreditFacilityRepayment,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) {
        self.events.push(CreditFacilityEvent::PaymentRecorded {
            tx_id: repayment.tx_id,
            tx_ref: repayment.tx_ref,
            disbursal_amount: repayment.amounts.disbursal,
            interest_amount: repayment.amounts.interest,
            audit_info: audit_info.clone(),
            recorded_in_ledger_at: recorded_at,
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, &audit_info);
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

    pub fn is_fully_collateralized(&self) -> bool {
        self.last_collateralization_state() == CollateralizationState::FullyCollateralized
    }

    pub fn maybe_update_collateralization(
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
                CreditFacilityStatus::Active | CreditFacilityStatus::Expired => {
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

        if let Some(calculated_collateralization) = collateralization_update {
            self.events
                .push(CreditFacilityEvent::CollateralizationChanged {
                    state: calculated_collateralization,
                    collateral: self.collateral(),
                    outstanding: self.outstanding(),
                    price,
                    recorded_at: Utc::now(),
                    audit_info: audit_info.clone(),
                });

            return Some(calculated_collateralization);
        }

        None
    }

    fn count_collateral_adjustments(&self) -> usize {
        self.events
            .iter_all()
            .filter(|event| matches!(event, CreditFacilityEvent::CollateralUpdated { .. }))
            .count()
    }

    pub(super) fn initiate_collateral_update(
        &self,
        updated_collateral: Satoshis,
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

        let tx_ref = format!(
            "{}-collateral-{}",
            self.id,
            self.count_collateral_adjustments() + 1
        );

        let tx_id = LedgerTxId::new();

        Ok(CreditFacilityCollateralUpdate {
            abs_diff: collateral,
            credit_facility_account_ids: self.account_ids,
            tx_ref,
            tx_id,
            action,
        })
    }

    pub(super) fn confirm_collateral_update(
        &mut self,
        CreditFacilityCollateralUpdate {
            tx_id,
            tx_ref,
            abs_diff,
            action,
            ..
        }: CreditFacilityCollateralUpdate,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) -> Option<CollateralizationState> {
        let mut total_collateral = self.collateral();
        total_collateral = match action {
            CollateralAction::Add => total_collateral + abs_diff,
            CollateralAction::Remove => total_collateral - abs_diff,
        };
        self.events.push(CreditFacilityEvent::CollateralUpdated {
            tx_id,
            tx_ref,
            total_collateral,
            abs_diff,
            action,
            recorded_in_ledger_at: executed_at,
            audit_info: audit_info.clone(),
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, &audit_info)
    }

    fn is_completed(&self) -> bool {
        self.events
            .iter_all()
            .any(|event| matches!(event, CreditFacilityEvent::Completed { .. }))
    }

    pub(super) fn initiate_completion(
        &self,
    ) -> Result<CreditFacilityCompletion, CreditFacilityError> {
        if self.is_completed() {
            return Err(CreditFacilityError::AlreadyCompleted);
        }
        if !self.outstanding().is_zero() {
            return Err(CreditFacilityError::OutstandingAmount);
        }

        Ok(CreditFacilityCompletion {
            tx_id: LedgerTxId::new(),
            tx_ref: format!("{}-completion", self.id),
            collateral: self.collateral(),
            credit_facility_account_ids: self.account_ids,
            customer_account_ids: self.customer_account_ids,
        })
    }

    pub(super) fn collateralization_ratio(&self) -> Option<Decimal> {
        let amount = if self.status() == CreditFacilityStatus::PendingCollateralization
            || self.status() == CreditFacilityStatus::PendingApproval
            || self.total_disbursed() == UsdCents::ZERO
        {
            self.initial_facility()
        } else {
            self.outstanding().total()
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

    pub(super) fn confirm_completion(
        &mut self,
        CreditFacilityCompletion {
            tx_id,
            tx_ref,
            collateral,
            ..
        }: CreditFacilityCompletion,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
    ) {
        self.confirm_collateral_update(
            CreditFacilityCollateralUpdate {
                credit_facility_account_ids: self.account_ids,
                tx_id,
                tx_ref,
                abs_diff: collateral,
                action: CollateralAction::Remove,
            },
            executed_at,
            audit_info.clone(),
            price,
            upgrade_buffer_cvl_pct,
        );

        self.events.push(CreditFacilityEvent::Completed {
            completed_at: executed_at,
            audit_info,
        });
    }

    pub(super) fn disbursal_amount_from_idx(&self, idx: DisbursalIdx) -> UsdCents {
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
                    customer_account_ids,
                    terms: t,
                    ..
                } => {
                    terms = Some(*t);
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .terms(*t)
                        .account_ids(*account_ids)
                        .customer_account_ids(*customer_account_ids)
                }
                CreditFacilityEvent::ApprovalProcessStarted {
                    approval_process_id,
                    ..
                } => builder = builder.approval_process_id(*approval_process_id),
                CreditFacilityEvent::Activated { activated_at, .. } => {
                    builder = builder.activated_at(*activated_at).expires_at(
                        terms
                            .expect("terms should be set")
                            .duration
                            .expiration_date(*activated_at),
                    )
                }
                CreditFacilityEvent::ApprovalProcessConcluded { .. } => (),
                CreditFacilityEvent::DisbursalInitiated { .. } => (),
                CreditFacilityEvent::DisbursalConcluded { .. } => (),
                CreditFacilityEvent::InterestAccrualStarted { .. } => (),
                CreditFacilityEvent::InterestAccrualConcluded { .. } => (),
                CreditFacilityEvent::CollateralUpdated { .. } => (),
                CreditFacilityEvent::CollateralizationChanged { .. } => (),
                CreditFacilityEvent::PaymentRecorded { .. } => (),
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
    customer_account_ids: CustomerLedgerAccountIds,
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
            [
                CreditFacilityEvent::Initialized {
                    id: self.id,
                    audit_info: self.audit_info.clone(),
                    customer_id: self.customer_id,
                    terms: self.terms,
                    facility: self.facility,
                    account_ids: self.account_ids,
                    customer_account_ids: self.customer_account_ids,
                },
                CreditFacilityEvent::ApprovalProcessStarted {
                    approval_process_id: self.approval_process_id,
                    audit_info: self.audit_info,
                },
            ],
        )
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::{
        audit::AuditEntryId,
        credit_facility::*,
        terms::{Duration, InterestInterval},
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .accrual_interval(InterestInterval::EndOfMonth)
            .incurrence_interval(InterestInterval::EndOfDay)
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
        vec![
            CreditFacilityEvent::Initialized {
                id: CreditFacilityId::new(),
                audit_info: dummy_audit_info(),
                customer_id: CustomerId::new(),
                facility: UsdCents::from(100),
                terms: default_terms(),
                account_ids: CreditFacilityAccountIds::new(),
                customer_account_ids: CustomerLedgerAccountIds::new(),
            },
            CreditFacilityEvent::ApprovalProcessStarted {
                approval_process_id: ApprovalProcessId::new(),
                audit_info: dummy_audit_info(),
            },
        ]
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
        events.push(CreditFacilityEvent::DisbursalInitiated {
            disbursal_id,
            approval_process_id: disbursal_id.into(),
            idx: first_idx,
            amount: UsdCents::ONE,
            audit_info: dummy_audit_info(),
        });
        assert!(matches!(
            facility_from(events.clone()).initiate_disbursal(
                UsdCents::ONE,
                Utc::now(),
                dummy_audit_info()
            ),
            Err(CreditFacilityError::DisbursalInProgress)
        ));

        events.push(CreditFacilityEvent::DisbursalConcluded {
            idx: first_idx,
            tx_id: Some(LedgerTxId::new()),
            recorded_at: Utc::now(),
            audit_info: dummy_audit_info(),
        });
        assert!(facility_from(events)
            .initiate_disbursal(UsdCents::ONE, Utc::now(), dummy_audit_info())
            .is_ok());
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
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.outstanding(),
            CreditFacilityReceivable {
                disbursed: UsdCents::from(100),
                interest: UsdCents::ZERO
            }
        );
    }

    #[test]
    fn outstanding_from_due_before_expiry() {
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
                recorded_at: activated_at,
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.outstanding_from_due().disbursed,
            UsdCents::ZERO
        );
    }

    #[test]
    fn outstanding_from_due_after_expiry() {
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
                recorded_at: activated_at,
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(events);

        assert_eq!(
            credit_facility.outstanding_from_due().disbursed,
            UsdCents::from(100)
        );
    }

    #[test]
    fn collateral() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(credit_facility.collateral(), Satoshis::ZERO);

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(credit_facility.collateral(), Satoshis::from(10000));

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(5000))
            .unwrap();
        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
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

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(500))
            .unwrap();
        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        assert_eq!(credit_facility.collateralization_ratio(), Some(dec!(5)));
    }

    #[test]
    fn collateralization_ratio_when_active_disbursal() {
        let mut events = initial_events();
        let disbursal_id = DisbursalId::new();
        events.extend([
            CreditFacilityEvent::CollateralUpdated {
                tx_id: LedgerTxId::new(),
                tx_ref: "tx-ref".to_string(),
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

        let first_period = credit_facility
            .next_interest_accrual_period()
            .unwrap()
            .unwrap();
        let InterestPeriod { start, .. } = first_period;
        assert_eq!(
            Utc::now().format("%Y-%m-%d").to_string(),
            start.format("%Y-%m-%d").to_string()
        );

        let new_accrual = credit_facility
            .start_interest_accrual(dummy_audit_info())
            .unwrap()
            .unwrap();
        let accrual = InterestAccrual::try_from_events(new_accrual.into_events()).unwrap();

        let second_period = credit_facility
            .next_interest_accrual_period()
            .unwrap()
            .unwrap();
        assert_eq!(first_period.next(), second_period);

        credit_facility.confirm_interest_accrual(
            CreditFacilityInterestAccrual {
                interest: UsdCents::ONE,
                tx_ref: "tx_ref".to_string(),
                tx_id: LedgerTxId::new(),
                accrued_at: accrual
                    .terms
                    .incurrence_interval
                    .period_from(accrual.started_at)
                    .end,
                credit_facility_account_ids: credit_facility.account_ids,
            },
            accrual.idx,
            dummy_audit_info(),
        );
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

        let new_accrual = credit_facility
            .start_interest_accrual(dummy_audit_info())
            .unwrap()
            .unwrap();
        let mut accrual = InterestAccrual::try_from_events(new_accrual.into_events()).unwrap();
        let mut next_accrual_period = credit_facility.next_interest_accrual_period().unwrap();
        while next_accrual_period.is_some() {
            credit_facility.confirm_interest_accrual(
                CreditFacilityInterestAccrual {
                    interest: UsdCents::ONE,
                    tx_ref: "tx_ref".to_string(),
                    tx_id: LedgerTxId::new(),
                    accrued_at: accrual
                        .terms
                        .incurrence_interval
                        .period_from(accrual.started_at)
                        .end,
                    credit_facility_account_ids: credit_facility.account_ids,
                },
                accrual.idx,
                dummy_audit_info(),
            );

            let new_idx = accrual.idx.next();
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
                .facility_expires_at(
                    credit_facility
                        .expires_at
                        .expect("Facility is already approved"),
                )
                .terms(credit_facility.terms)
                .audit_info(dummy_audit_info())
                .build()
                .unwrap();
            accrual = InterestAccrual::try_from_events(new_accrual.into_events()).unwrap();

            next_accrual_period = credit_facility.next_interest_accrual_period().unwrap();
        }
        assert_eq!(
            accrual.started_at.format("%Y-%m").to_string(),
            credit_facility
                .expires_at
                .unwrap()
                .format("%Y-%m")
                .to_string()
        );
    }

    #[test]
    fn cvl_is_approval_allowed() {
        let terms = default_terms();

        let facility_cvl = FacilityCVL {
            total: terms.margin_call_cvl - CVLPct::from(dec!(1)),
            disbursed: CVLPct::ZERO,
        };
        assert!(matches!(
            facility_cvl.is_approval_allowed(terms),
            Err(CreditFacilityError::BelowMarginLimit),
        ));

        let facility_cvl = FacilityCVL {
            total: terms.margin_call_cvl,
            disbursed: CVLPct::ZERO,
        };
        assert!(matches!(facility_cvl.is_approval_allowed(terms), Ok(())));
    }

    #[test]
    fn check_activated_at() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(credit_facility.activated_at, None);
        assert_eq!(credit_facility.expires_at, None);

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        credit_facility.confirm_collateral_update(
            credit_facility_collateral_update,
            Utc::now(),
            dummy_audit_info(),
            default_price(),
            default_upgrade_buffer_cvl_pct(),
        );
        let approval_time = Utc::now();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let credit_facility_approval = credit_facility.activation_data(default_price()).unwrap();
        credit_facility.activate(credit_facility_approval, approval_time, dummy_audit_info());
        assert_eq!(credit_facility.activated_at, Some(approval_time));
        assert!(credit_facility.expires_at.is_some())
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
        let credit_facility = facility_from(events);

        let res = credit_facility.activation_data(default_price());
        assert!(matches!(res, Err(CreditFacilityError::NoCollateral)));
    }

    #[test]
    fn reject_credit_facility_activate_below_margin_limit() {
        let mut credit_facility = facility_from(initial_events());

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(100))
            .unwrap();
        credit_facility
            .confirm_collateral_update(
                credit_facility_collateral_update,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            )
            .unwrap();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let first_approval = credit_facility.activation_data(default_price());
        assert!(matches!(
            first_approval,
            Err(CreditFacilityError::BelowMarginLimit)
        ));
    }

    #[test]
    fn status() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(
            credit_facility.status(),
            CreditFacilityStatus::PendingCollateralization
        );

        let credit_facility_collateral_update = credit_facility
            .initiate_collateral_update(Satoshis::from(10000))
            .unwrap();
        credit_facility
            .confirm_collateral_update(
                credit_facility_collateral_update,
                Utc::now(),
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
        let credit_facility_approval = credit_facility.activation_data(default_price()).unwrap();
        credit_facility.activate(credit_facility_approval, Utc::now(), dummy_audit_info());
        assert_eq!(credit_facility.status(), CreditFacilityStatus::Active);
    }

    mod activation_data {
        use super::*;

        #[test]
        fn errors_when_not_approved_yet() {
            let credit_facility = facility_from(initial_events());
            assert!(matches!(
                credit_facility.activation_data(default_price()),
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
            let credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activation_data(default_price()),
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
            let credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activation_data(default_price()),
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
                    tx_ref: "".to_string(),
                    total_collateral: Satoshis::ONE,
                    abs_diff: Satoshis::ONE,
                    action: CollateralAction::Add,
                    recorded_in_ledger_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
            ]);
            let credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activation_data(default_price()),
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
                    tx_ref: "".to_string(),
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
            let credit_facility = facility_from(events);

            assert!(matches!(
                credit_facility.activation_data(default_price()),
                Err(CreditFacilityError::AlreadyActivated)
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
                    tx_ref: "".to_string(),
                    total_collateral: collateral_amount,
                    abs_diff: collateral_amount,
                    action: CollateralAction::Add,
                    recorded_in_ledger_at: Utc::now(),
                    audit_info: dummy_audit_info(),
                },
            ]);
            let credit_facility = facility_from(events);

            assert!(credit_facility.activation_data(default_price()).is_ok(),);
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
                .customer_account_ids(CustomerLedgerAccountIds::new())
                .audit_info(dummy_audit_info())
                .build()
                .expect("could not build new credit facility");
            let mut credit_facility =
                CreditFacility::try_from_events(new_credit_facility.into_events()).unwrap();

            let credit_facility_collateral_update = credit_facility
                .initiate_collateral_update(Satoshis::from(50_00_000_000))
                .unwrap();
            credit_facility
                .confirm_collateral_update(
                    credit_facility_collateral_update,
                    facility_activated_at,
                    dummy_audit_info(),
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                )
                .unwrap();

            credit_facility
                .approval_process_concluded(true, dummy_audit_info())
                .unwrap();
            let credit_facility_approval =
                credit_facility.activation_data(default_price()).unwrap();
            credit_facility.activate(
                credit_facility_approval,
                facility_activated_at,
                dummy_audit_info(),
            );

            let new_disbursal = credit_facility
                .initiate_disbursal(
                    UsdCents::from(600_000_00),
                    facility_activated_at,
                    dummy_audit_info(),
                )
                .unwrap();
            let mut disbursal = Disbursal::try_from_events(new_disbursal.into_events()).unwrap();
            disbursal
                .approval_process_concluded(true, dummy_audit_info())
                .unwrap();
            let disbursal_data = disbursal.disbursal_data().unwrap();
            disbursal.confirm(&disbursal_data, facility_activated_at, dummy_audit_info());
            credit_facility.confirm_disbursal(
                &disbursal,
                Some(disbursal_data.tx_id),
                facility_activated_at,
                dummy_audit_info(),
            );

            let new_accrual = credit_facility
                .start_interest_accrual(dummy_audit_info())
                .unwrap()
                .unwrap();
            let mut accrual = InterestAccrual::try_from_events(new_accrual.into_events()).unwrap();
            let mut accrual_data: Option<CreditFacilityInterestAccrual> = None;
            while accrual_data.is_none() {
                let interest_incurrence = accrual.initiate_incurrence(
                    credit_facility.outstanding(),
                    credit_facility.account_ids,
                );
                accrual_data = accrual.confirm_incurrence(interest_incurrence, dummy_audit_info());
            }
            let accrual_data = accrual_data.unwrap();
            accrual.confirm_accrual(accrual_data.clone(), dummy_audit_info());
            credit_facility.confirm_interest_accrual(accrual_data, accrual.idx, dummy_audit_info());

            credit_facility
        }

        #[test]
        fn initiate_repayment_errors_when_no_disbursals() {
            let credit_facility = facility_from(initial_events());

            let repayment_amount = UsdCents::from(5);
            assert!(credit_facility
                .initiate_repayment(repayment_amount)
                .is_err());
        }

        #[test]
        fn initiate_repayment_before_expiry_errors_for_amount_above_interest() {
            let activated_at = Utc::now();
            let credit_facility = credit_facility_with_interest_accrual(activated_at);
            let interest = credit_facility.outstanding().interest;

            assert!(credit_facility
                .initiate_repayment(interest + UsdCents::ONE)
                .is_err());
            assert!(credit_facility.initiate_repayment(interest).is_ok());
        }

        #[test]
        fn initiate_repayment_after_expiry_errors_for_amount_above_total() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let credit_facility = credit_facility_with_interest_accrual(activated_at);
            let outstanding = credit_facility.outstanding().total();

            assert!(credit_facility
                .initiate_repayment(outstanding + UsdCents::ONE)
                .is_err());
            assert!(credit_facility.initiate_repayment(outstanding).is_ok());
        }

        #[test]
        fn confirm_repayment_before_expiry() {
            let activated_at = Utc::now();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let repayment_amount = credit_facility.outstanding().interest;
            let repayment = credit_facility
                .initiate_repayment(repayment_amount)
                .unwrap();
            let outstanding_before = credit_facility.outstanding();

            credit_facility.confirm_repayment(
                repayment,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let outstanding_after = credit_facility.outstanding();

            assert_eq!(
                outstanding_before.total() - outstanding_after.total(),
                repayment_amount
            );
        }

        #[test]
        fn confirm_partial_repayment_after_expiry() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let partial_repayment_amount = credit_facility.outstanding().interest
                + credit_facility.outstanding().disbursed
                - UsdCents::from(100);
            let repayment = credit_facility
                .initiate_repayment(partial_repayment_amount)
                .unwrap();
            let outstanding_before = credit_facility.outstanding();

            credit_facility.confirm_repayment(
                repayment,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            let outstanding_after = credit_facility.outstanding();

            assert!(!outstanding_after.is_zero());
            assert_eq!(
                outstanding_before.total() - outstanding_after.total(),
                partial_repayment_amount
            );
        }

        #[test]
        fn confirm_completion() {
            let activated_at = "2023-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let mut credit_facility = credit_facility_with_interest_accrual(activated_at);

            let repayment = credit_facility
                .initiate_repayment(credit_facility.outstanding().total())
                .unwrap();
            credit_facility.confirm_repayment(
                repayment,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            assert!(credit_facility.outstanding().is_zero());

            let completion = credit_facility.initiate_completion().unwrap();
            credit_facility.confirm_completion(
                completion,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );
            assert!(credit_facility.is_completed());
            assert!(credit_facility.status() == CreditFacilityStatus::Closed);
        }
    }
}
