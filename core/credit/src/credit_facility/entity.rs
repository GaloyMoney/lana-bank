use chrono::{DateTime, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::{
    interest_accrual_cycle::*,
    ledger::*,
    obligation::{NewObligation, ObligationsAmounts},
    primitives::*,
    terms::{InterestPeriod, TermValues},
};

use super::error::CreditFacilityError;

#[allow(clippy::large_enum_variant)]
#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "CreditFacilityId")]
pub enum CreditFacilityEvent {
    Initialized {
        id: CreditFacilityId,
        customer_id: CustomerId,
        collateral_id: CollateralId,
        ledger_tx_id: LedgerTxId,
        terms: TermValues,
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
    InterestAccrualCycleStarted {
        interest_accrual_id: InterestAccrualCycleId,
        interest_accrual_cycle_idx: InterestAccrualCycleIdx,
        interest_period: InterestPeriod,
        audit_info: AuditInfo,
    },
    InterestAccrualCycleConcluded {
        interest_accrual_cycle_idx: InterestAccrualCycleIdx,
        ledger_tx_id: LedgerTxId,
        obligation_id: ObligationId,
        audit_info: AuditInfo,
    },
    CollateralizationStateChanged {
        collateralization_state: CollateralizationState,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        price: PriceOfOneBTC,
        audit_info: AuditInfo,
    },
    CollateralizationRatioChanged {
        collateralization_ratio: Option<Decimal>,
        audit_info: AuditInfo,
    },
    Completed {
        audit_info: AuditInfo,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl From<CreditFacilityBalanceSummary> for CreditFacilityReceivable {
    fn from(balance: CreditFacilityBalanceSummary) -> Self {
        Self {
            disbursed: balance.disbursed_outstanding_payable(),
            interest: balance.interest_outstanding_payable(),
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

    pub fn is_zero(&self) -> bool {
        self.total().is_zero()
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
    pub collateral_id: CollateralId,
    pub amount: UsdCents,
    pub terms: TermValues,
    pub account_ids: CreditFacilityAccountIds,
    pub disbursal_credit_account_id: CalaAccountId,
    #[builder(setter(strip_option), default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub matures_at: Option<DateTime<Utc>>,

    #[es_entity(nested)]
    #[builder(default)]
    interest_accruals: Nested<InterestAccrualCycle>,
    events: EntityEvents<CreditFacilityEvent>,
}

impl CreditFacility {
    pub fn creation_data(&self) -> CreditFacilityCreation {
        self.events
            .iter_all()
            .find_map(|event| match event {
                CreditFacilityEvent::Initialized {
                    ledger_tx_id,
                    account_ids,
                    amount,
                    ..
                } => Some(CreditFacilityCreation {
                    tx_id: *ledger_tx_id,
                    tx_ref: format!("{}-create", self.id),
                    credit_facility_account_ids: *account_ids,
                    facility_amount: *amount,
                }),
                _ => None,
            })
            .expect("Facility was not Initialized")
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn structuring_fee(&self) -> UsdCents {
        self.terms.one_time_fee_rate.apply(self.amount)
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
                    return Ok(*approved);
                }
                _ => continue,
            }
        }
        Err(CreditFacilityError::ApprovalInProgress)
    }

    pub fn is_activated(&self) -> bool {
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

        if !self.terms.is_approval_allowed(balances, price) {
            return Err(CreditFacilityError::BelowMarginLimit);
        }

        self.activated_at = Some(activated_at);
        self.matures_at = Some(self.terms.duration.maturity_date(activated_at));
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

    pub(crate) fn check_disbursal_date(&self, initiated_at: DateTime<Utc>) -> bool {
        initiated_at < self.matures_at.expect("Facility not activated yet")
    }

    fn next_interest_accrual_cycle_period(
        &self,
    ) -> Result<Option<InterestPeriod>, CreditFacilityError> {
        let last_accrual_start_date = self.events.iter_all().rev().find_map(|event| match event {
            CreditFacilityEvent::InterestAccrualCycleStarted {
                interest_period: period,
                ..
            } => Some(period.start),
            _ => None,
        });

        let interval = self.terms.accrual_cycle_interval;
        let full_period = match last_accrual_start_date {
            Some(last_accrual_start_date) => interval.period_from(last_accrual_start_date).next(),
            None => interval.period_from(
                self.activated_at
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
                CreditFacilityEvent::InterestAccrualCycleStarted {
                    interest_accrual_cycle_idx: idx,
                    ..
                } => Some(idx.next()),
                _ => None,
            })
            .unwrap_or(InterestAccrualCycleIdx::FIRST);
        let id = InterestAccrualCycleId::new();
        self.events
            .push(CreditFacilityEvent::InterestAccrualCycleStarted {
                interest_accrual_id: id,
                interest_accrual_cycle_idx: idx,
                interest_period: accrual_cycle_period,
                audit_info: audit_info.clone(),
            });

        let new_accrual = NewInterestAccrualCycle::builder()
            .id(id)
            .credit_facility_id(self.id)
            .account_ids(self.account_ids.into())
            .idx(idx)
            .period(accrual_cycle_period)
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
    ) -> Result<Idempotent<NewObligation>, CreditFacilityError> {
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
                match accrual.record_accrual_cycle(accrual_cycle_data.clone(), audit_info.clone()) {
                    Idempotent::Executed(new_obligation) => new_obligation,
                    Idempotent::Ignored => {
                        return Ok(Idempotent::Ignored);
                    }
                },
            )
        };
        self.events
            .push(CreditFacilityEvent::InterestAccrualCycleConcluded {
                interest_accrual_cycle_idx: idx,
                obligation_id: new_obligation.id,
                ledger_tx_id: accrual_cycle_data.tx_id,
                audit_info: audit_info.clone(),
            });

        Ok(Idempotent::Executed(new_obligation))
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

    pub fn last_collateralization_state(&self) -> CollateralizationState {
        if self.is_completed() {
            return CollateralizationState::NoCollateral;
        }

        self.events
            .iter_all()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::CollateralizationStateChanged {
                    collateralization_state: state,
                    ..
                } => Some(*state),
                _ => None,
            })
            .unwrap_or(CollateralizationState::NoCollateral)
    }

    pub fn last_collateralization_ratio(&self) -> Option<Decimal> {
        self.events.iter_all().rev().find_map(|event| match event {
            CreditFacilityEvent::CollateralizationRatioChanged {
                collateralization_ratio: ratio,
                ..
            } => *ratio,
            _ => None,
        })
    }

    fn is_fully_collateralized(&self) -> bool {
        self.last_collateralization_state() == CollateralizationState::FullyCollateralized
    }

    pub(crate) fn update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
        audit_info: &AuditInfo,
    ) -> Idempotent<Option<CollateralizationState>> {
        let ratio_changed = self
            .update_collateralization_ratio(&balances, audit_info.clone())
            .did_execute();

        let last_collateralization_state = self.last_collateralization_state();

        let collateralization_update = match self.status() {
            CreditFacilityStatus::PendingCollateralization
            | CreditFacilityStatus::PendingApproval => self.terms.collateralization_update(
                balances.facility_amount_cvl(price),
                last_collateralization_state,
                None,
                true,
            ),
            CreditFacilityStatus::Active | CreditFacilityStatus::Matured => {
                self.terms.collateralization_update(
                    balances.current_cvl(price),
                    last_collateralization_state,
                    Some(upgrade_buffer_cvl_pct),
                    false,
                )
            }
            CreditFacilityStatus::Closed => Some(CollateralizationState::NoCollateral),
        };

        if let Some(calculated_collateralization) = collateralization_update {
            self.events
                .push(CreditFacilityEvent::CollateralizationStateChanged {
                    collateralization_state: calculated_collateralization,
                    collateral: balances.collateral(),
                    outstanding: balances.into(),
                    price,
                    audit_info: audit_info.clone(),
                });

            Idempotent::Executed(Some(calculated_collateralization))
        } else if ratio_changed {
            Idempotent::Executed(None)
        } else {
            Idempotent::Ignored
        }
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
        _price: PriceOfOneBTC,
        _upgrade_buffer_cvl_pct: CVLPct,
        balances: CreditFacilityBalanceSummary,
    ) -> Result<Idempotent<CreditFacilityCompletion>, CreditFacilityError> {
        idempotency_guard!(
            self.events.iter_all(),
            CreditFacilityEvent::Completed { .. }
        );
        if balances.any_outstanding_or_defaulted() {
            return Err(CreditFacilityError::OutstandingAmount);
        }

        let res = CreditFacilityCompletion {
            tx_id: LedgerTxId::new(),
            collateral: balances.collateral(),
            credit_facility_account_ids: self.account_ids,
        };

        self.events
            .push(CreditFacilityEvent::Completed { audit_info });

        Ok(Idempotent::Executed(res))
    }

    fn update_collateralization_ratio(
        &mut self,
        balance: &CreditFacilityBalanceSummary,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        let ratio = balance.current_collateralization_ratio();

        if self.last_collateralization_ratio() != ratio {
            self.events
                .push(CreditFacilityEvent::CollateralizationRatioChanged {
                    collateralization_ratio: ratio,
                    audit_info,
                });
        } else {
            return Idempotent::Ignored;
        }

        Idempotent::Executed(())
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
                    collateral_id,
                    account_ids,
                    disbursal_credit_account_id,
                    terms: t,
                    approval_process_id,
                    ..
                } => {
                    terms = Some(*t);
                    builder = builder
                        .id(*id)
                        .amount(*amount)
                        .customer_id(*customer_id)
                        .collateral_id(*collateral_id)
                        .terms(*t)
                        .account_ids(*account_ids)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .approval_process_id(*approval_process_id)
                }
                CreditFacilityEvent::Activated { activated_at, .. } => {
                    let matures_at = terms
                        .expect("terms should be set")
                        .duration
                        .maturity_date(*activated_at);
                    builder = builder.activated_at(*activated_at).matures_at(matures_at)
                }
                CreditFacilityEvent::ApprovalProcessConcluded { .. } => (),
                CreditFacilityEvent::InterestAccrualCycleStarted { .. } => (),
                CreditFacilityEvent::InterestAccrualCycleConcluded { .. } => (),
                CreditFacilityEvent::CollateralizationStateChanged { .. } => (),
                CreditFacilityEvent::CollateralizationRatioChanged { .. } => (),
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
    pub(super) ledger_tx_id: LedgerTxId,
    #[builder(setter(into))]
    pub(super) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) collateral_id: CollateralId,
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
                ledger_tx_id: self.ledger_tx_id,
                audit_info: self.audit_info.clone(),
                customer_id: self.customer_id,
                collateral_id: self.collateral_id,
                terms: self.terms,
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
        terms::{FacilityDuration, InterestInterval, OneTimeFeeRatePct},
        *,
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(FacilityDuration::Months(3))
            .interest_due_duration_from_accrual(ObligationDuration::Days(0))
            .obligation_overdue_duration_from_due(None)
            .obligation_liquidation_duration_from_due(None)
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

    fn default_balances(facility: UsdCents) -> CreditFacilityBalanceSummary {
        CreditFacilityBalanceSummary {
            facility,
            facility_remaining: facility,
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
            ledger_tx_id: LedgerTxId::new(),
            audit_info: dummy_audit_info(),
            customer_id: CustomerId::new(),
            collateral_id: CollateralId::new(),
            amount: default_facility(),
            terms: default_terms(),
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
            .period_from(credit_facility.activated_at.expect("Not activated"));
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

            let id = InterestAccrualCycleId::new();
            credit_facility
                .events
                .push(CreditFacilityEvent::InterestAccrualCycleStarted {
                    interest_accrual_id: id,
                    interest_accrual_cycle_idx: new_idx,
                    interest_period: next_accrual_period.unwrap(),
                    audit_info: dummy_audit_info(),
                });
            let new_accrual = NewInterestAccrualCycle::builder()
                .id(id)
                .credit_facility_id(credit_facility.id)
                .account_ids(credit_facility.account_ids.into())
                .idx(new_idx)
                .period(next_accrual_period.unwrap())
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
    fn check_activated_at() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(credit_facility.activated_at, None);
        assert_eq!(credit_facility.matures_at, None);

        let approval_time = Utc::now();

        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let mut balances = default_balances(credit_facility.amount);
        balances.collateral = default_full_collateral();

        assert!(
            credit_facility
                .activate(approval_time, default_price(), balances, dummy_audit_info())
                .unwrap()
                .did_execute()
        );
        assert_eq!(credit_facility.activated_at, Some(approval_time));
        assert!(credit_facility.matures_at.is_some())
    }

    #[test]
    fn status() {
        let mut credit_facility = facility_from(initial_events());
        assert_eq!(
            credit_facility.status(),
            CreditFacilityStatus::PendingCollateralization
        );

        credit_facility
            .update_collateralization(
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                default_balances(credit_facility.amount).with_collateral(default_full_collateral()),
                &dummy_audit_info(),
            )
            .unwrap();

        assert_eq!(
            credit_facility.status(),
            CreditFacilityStatus::PendingApproval
        );
        credit_facility
            .approval_process_concluded(true, dummy_audit_info())
            .unwrap();
        let mut balances = default_balances(credit_facility.amount);
        balances.collateral = default_full_collateral();
        assert!(
            credit_facility
                .activate(Utc::now(), default_price(), balances, dummy_audit_info())
                .unwrap()
                .did_execute()
        );
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
                Err(CreditFacilityError::BelowMarginLimit)
            ));
        }

        #[test]
        fn errors_if_collateral_below_margin() {
            let mut events = initial_events();
            events.extend([CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            }]);
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
            events.extend([CreditFacilityEvent::ApprovalProcessConcluded {
                approval_process_id: ApprovalProcessId::new(),
                approved: true,
                audit_info: dummy_audit_info(),
            }]);
            let mut credit_facility = facility_from(events);
            let mut balances = default_balances(credit_facility.amount);
            balances.collateral = collateral_amount;

            assert!(
                credit_facility
                    .activate(Utc::now(), default_price(), balances, dummy_audit_info())
                    .is_ok()
            );
        }
    }

    mod completion {
        use super::*;

        impl From<CreditFacilityReceivable> for ObligationsAmounts {
            fn from(receivable: CreditFacilityReceivable) -> Self {
                Self {
                    disbursed: receivable.disbursed,
                    interest: receivable.interest,
                }
            }
        }

        #[test]
        fn can_complete() {
            let mut credit_facility = facility_from(initial_events());

            let _ = credit_facility
                .complete(
                    dummy_audit_info(),
                    default_price(),
                    default_upgrade_buffer_cvl_pct(),
                    CreditFacilityBalanceSummary {
                        collateral: Satoshis::ZERO,
                        not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                        due_disbursed_outstanding: UsdCents::ZERO,
                        overdue_disbursed_outstanding: UsdCents::ZERO,
                        disbursed_defaulted: UsdCents::ZERO,
                        not_yet_due_interest_outstanding: UsdCents::ZERO,
                        due_interest_outstanding: UsdCents::ZERO,
                        overdue_interest_outstanding: UsdCents::ZERO,
                        interest_defaulted: UsdCents::ZERO,

                        facility: UsdCents::from(2),
                        facility_remaining: UsdCents::from(1),
                        disbursed: UsdCents::from(1),
                        interest_posted: UsdCents::from(1),
                    },
                )
                .unwrap();
            assert!(credit_facility.is_completed());
            assert!(credit_facility.status() == CreditFacilityStatus::Closed);
        }

        #[test]
        fn errors_if_not_yet_due_outstanding() {
            let mut credit_facility = facility_from(initial_events());

            let res_disbursed = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    not_yet_due_disbursed_outstanding: UsdCents::from(1),
                    not_yet_due_interest_outstanding: UsdCents::ZERO,

                    collateral: Satoshis::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_disbursed,
                Err(CreditFacilityError::OutstandingAmount)
            ));

            let res_interest = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::from(1),

                    collateral: Satoshis::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_interest,
                Err(CreditFacilityError::OutstandingAmount)
            ));
        }

        #[test]
        fn errors_if_due_outstanding() {
            let mut credit_facility = facility_from(initial_events());

            let res_disbursed = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    due_disbursed_outstanding: UsdCents::from(1),
                    due_interest_outstanding: UsdCents::ZERO,

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_disbursed,
                Err(CreditFacilityError::OutstandingAmount)
            ));

            let res_interest = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    due_disbursed_outstanding: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::from(1),

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_interest,
                Err(CreditFacilityError::OutstandingAmount)
            ));
        }

        #[test]
        fn errors_if_overdue_outstanding() {
            let mut credit_facility = facility_from(initial_events());

            let res_disbursed = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    overdue_disbursed_outstanding: UsdCents::from(1),
                    overdue_interest_outstanding: UsdCents::ZERO,

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_disbursed,
                Err(CreditFacilityError::OutstandingAmount)
            ));

            let res_interest = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::from(1),

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    disbursed_defaulted: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    interest_defaulted: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_interest,
                Err(CreditFacilityError::OutstandingAmount)
            ));
        }

        #[test]
        fn errors_if_defaulted_outstanding() {
            let mut credit_facility = facility_from(initial_events());

            let res_disbursed = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    disbursed_defaulted: UsdCents::from(1),
                    interest_defaulted: UsdCents::ZERO,

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_disbursed,
                Err(CreditFacilityError::OutstandingAmount)
            ));

            let res_interest = credit_facility.complete(
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
                CreditFacilityBalanceSummary {
                    disbursed_defaulted: UsdCents::ZERO,
                    interest_defaulted: UsdCents::from(1),

                    collateral: Satoshis::ZERO,
                    not_yet_due_disbursed_outstanding: UsdCents::ZERO,
                    due_disbursed_outstanding: UsdCents::ZERO,
                    overdue_disbursed_outstanding: UsdCents::ZERO,
                    not_yet_due_interest_outstanding: UsdCents::ZERO,
                    due_interest_outstanding: UsdCents::ZERO,
                    overdue_interest_outstanding: UsdCents::ZERO,

                    facility: UsdCents::from(2),
                    facility_remaining: UsdCents::from(1),
                    disbursed: UsdCents::from(1),
                    interest_posted: UsdCents::from(1),
                },
            );
            assert!(matches!(
                res_interest,
                Err(CreditFacilityError::OutstandingAmount)
            ));
        }
    }
}
