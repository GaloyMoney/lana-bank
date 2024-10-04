use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    entity::*,
    ledger::{credit_facility::*, customer::CustomerLedgerAccountIds},
    primitives::*,
    terms::{CVLPct, CollateralizationState, TermValues},
};

use super::{disbursement::*, CreditFacilityCollateralUpdate, CreditFacilityError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
    ApprovalAdded {
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    Approved {
        tx_id: LedgerTxId,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    DisbursementInitiated {
        idx: DisbursementIdx,
        amount: UsdCents,
        audit_info: AuditInfo,
    },
    DisbursementConcluded {
        idx: DisbursementIdx,
        tx_id: LedgerTxId,
        recorded_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
    CollateralUpdated {
        tx_id: LedgerTxId,
        tx_ref: String,
        total_collateral: Satoshis,
        abs_diff: Satoshis,
        action: CollateralAction,
        audit_info: AuditInfo,
        recorded_in_ledger_at: DateTime<Utc>,
    },
    CollateralizationChanged {
        state: CollateralizationState,
        collateral: Satoshis,
        outstanding: CreditFacilityReceivable,
        price: PriceOfOneBTC,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    PaymentRecorded {
        tx_id: LedgerTxId,
        tx_ref: String,
        disbursement_amount: UsdCents,
        interest_amount: UsdCents,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

impl EntityEvent for CreditFacilityEvent {
    type EntityId = CreditFacilityId;
    fn event_table_name() -> &'static str {
        "credit_facility_events"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditFacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl CreditFacilityReceivable {
    pub fn total(&self) -> UsdCents {
        self.interest + self.disbursed
    }

    fn allocate_payment(
        &self,
        amount: UsdCents,
    ) -> Result<CreditFacilityPaymentAmounts, CreditFacilityError> {
        let mut remaining = amount;

        let interest = std::cmp::min(amount, self.interest);
        remaining -= interest;

        let disbursement = std::cmp::min(remaining, self.disbursed);
        remaining -= disbursement;

        Ok(CreditFacilityPaymentAmounts {
            interest,
            disbursement,
        })
    }
}

pub struct FacilityCVL {
    total: CVLPct,
    disbursed: CVLPct,
}

impl FacilityCVL {
    pub const ZERO: Self = Self {
        total: CVLPct::ZERO,
        disbursed: CVLPct::ZERO,
    };

    fn is_approval_allowed(&self, terms: TermValues) -> Result<(), CreditFacilityError> {
        if self.total < terms.margin_call_cvl {
            return Err(CreditFacilityError::BelowMarginLimit);
        }
        Ok(())
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct CreditFacility {
    pub id: CreditFacilityId,
    pub customer_id: CustomerId,
    pub terms: TermValues,
    pub account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(strip_option), default)]
    pub approved_at: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub expires_at: Option<DateTime<Utc>>,
    pub(super) events: EntityEvents<CreditFacilityEvent>,
}

impl Entity for CreditFacility {
    type Event = CreditFacilityEvent;
}

impl CreditFacility {
    fn initial_facility(&self) -> UsdCents {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::Initialized { facility, .. } => return *facility,
                _ => continue,
            }
        }
        UsdCents::ZERO
    }

    fn total_disbursed(&self) -> UsdCents {
        let mut amounts = std::collections::HashMap::new();
        self.events
            .iter()
            .fold(UsdCents::from(0), |mut total_sum, event| {
                match event {
                    CreditFacilityEvent::DisbursementInitiated { idx, amount, .. } => {
                        amounts.insert(*idx, *amount);
                    }
                    CreditFacilityEvent::DisbursementConcluded { idx, .. } => {
                        if let Some(amount) = amounts.remove(idx) {
                            total_sum += amount;
                        }
                    }
                    _ => (),
                }
                total_sum
            })
    }

    fn facility_remaining(&self) -> UsdCents {
        self.initial_facility() - self.total_disbursed()
    }

    fn interest_accrued(&self) -> UsdCents {
        // TODO: implement
        UsdCents::ZERO
    }

    fn disbursed_payments(&self) -> UsdCents {
        // TODO: implement
        UsdCents::ZERO
    }

    fn interest_payments(&self) -> UsdCents {
        // TODO: implement
        UsdCents::ZERO
    }

    pub(super) fn is_approved(&self) -> bool {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::Approved { .. } => return true,
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
        if self.is_expired() {
            CreditFacilityStatus::Closed
        } else if self.is_approved() {
            CreditFacilityStatus::Active
        } else {
            CreditFacilityStatus::New
        }
    }

    fn approval_threshold_met(&self) -> bool {
        let mut n_admin = 0;
        let mut n_bank_manager = 0;

        for event in self.events.iter() {
            if let CreditFacilityEvent::ApprovalAdded {
                approving_user_roles,
                ..
            } = event
            {
                if approving_user_roles.contains(&Role::Superuser) {
                    return true;
                } else if approving_user_roles.contains(&Role::Admin) {
                    n_admin += 1;
                } else {
                    n_bank_manager += 1;
                }
            }
        }

        n_admin >= 1 && n_admin + n_bank_manager >= 2
    }

    fn has_user_previously_approved(&self, user_id: UserId) -> bool {
        for event in self.events.iter() {
            match event {
                CreditFacilityEvent::ApprovalAdded {
                    approving_user_id, ..
                } => {
                    if user_id == *approving_user_id {
                        return true;
                    }
                }
                _ => continue,
            }
        }
        false
    }

    pub(super) fn add_approval(
        &mut self,
        approving_user_id: UserId,
        approving_user_roles: HashSet<Role>,
        audit_info: AuditInfo,
        price: PriceOfOneBTC,
    ) -> Result<Option<CreditFacilityApprovalData>, CreditFacilityError> {
        if self.has_user_previously_approved(approving_user_id) {
            return Err(CreditFacilityError::UserCannotApproveTwice);
        }

        if self.is_approved() {
            return Err(CreditFacilityError::AlreadyApproved);
        }

        if self.collateral() == Satoshis::ZERO {
            return Err(CreditFacilityError::NoCollateral);
        }

        self.facility_cvl(price).is_approval_allowed(self.terms)?;

        self.events.push(CreditFacilityEvent::ApprovalAdded {
            approving_user_id,
            approving_user_roles,
            audit_info,
            recorded_at: Utc::now(),
        });

        if self.approval_threshold_met() {
            let tx_ref = format!("{}-approval", self.id);
            Ok(Some(CreditFacilityApprovalData {
                facility: self.initial_facility(),
                tx_ref,
                tx_id: LedgerTxId::new(),
                credit_facility_account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) fn confirm_approval(
        &mut self,
        CreditFacilityApprovalData { tx_id, .. }: CreditFacilityApprovalData,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.approved_at = Some(executed_at);
        self.expires_at = Some(self.terms.duration.expiration_date(executed_at));
        self.events.push(CreditFacilityEvent::Approved {
            tx_id,
            audit_info,
            recorded_at: executed_at,
        });
    }

    pub(super) fn initiate_disbursement(
        &mut self,
        audit_info: AuditInfo,
        amount: UsdCents,
    ) -> Result<NewDisbursement, CreditFacilityError> {
        if self.is_expired() {
            return Err(CreditFacilityError::AlreadyExpired);
        }

        if self.is_disbursement_in_progress() {
            return Err(CreditFacilityError::DisbursementInProgress);
        }

        let idx = self
            .events
            .iter()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::DisbursementInitiated { idx, .. } => Some(idx.next()),
                _ => None,
            })
            .unwrap_or(DisbursementIdx::FIRST);

        self.events
            .push(CreditFacilityEvent::DisbursementInitiated {
                idx,
                amount,
                audit_info,
            });

        Ok(NewDisbursement::builder()
            .id(DisbursementId::new())
            .facility_id(self.id)
            .idx(idx)
            .amount(amount)
            .account_ids(self.account_ids)
            .customer_account_ids(self.customer_account_ids)
            .audit_info(audit_info)
            .build()
            .expect("could not build new disbursement"))
    }

    pub(super) fn confirm_disbursement(
        &mut self,
        disbursement: &Disbursement,
        tx_id: LedgerTxId,
        executed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    ) {
        self.events
            .push(CreditFacilityEvent::DisbursementConcluded {
                idx: disbursement.idx,
                recorded_at: executed_at,
                tx_id,
                audit_info,
            });
    }

    fn is_disbursement_in_progress(&self) -> bool {
        for event in self.events.iter().rev() {
            if let CreditFacilityEvent::DisbursementInitiated { .. } = event {
                return true;
            }
            if let CreditFacilityEvent::DisbursementConcluded { .. } = event {
                return false;
            }
        }

        false
    }

    pub fn outstanding(&self) -> CreditFacilityReceivable {
        CreditFacilityReceivable {
            disbursed: self.total_disbursed() - self.disbursed_payments(),
            interest: self.interest_accrued() - self.interest_payments(),
        }
    }

    pub fn collateral(&self) -> Satoshis {
        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::CollateralUpdated {
                    total_collateral, ..
                } => Some(*total_collateral),
                _ => None,
            })
            .unwrap_or(Satoshis::ZERO)
    }

    pub fn facility_cvl(&self, price: PriceOfOneBTC) -> FacilityCVL {
        let collateral_value = price.sats_to_cents_round_down(self.collateral());

        FacilityCVL {
            total: CVLPct::from_loan_amounts(
                collateral_value,
                self.outstanding().total() + self.facility_remaining(),
            ),
            disbursed: CVLPct::from_loan_amounts(collateral_value, self.outstanding().total()),
        }
    }

    pub(super) fn initiate_repayment(
        &self,
        amount: UsdCents,
    ) -> Result<CreditFacilityRepayment, CreditFacilityError> {
        let outstanding = self.outstanding();

        if outstanding.total() < amount {
            return Err(
                CreditFacilityError::PaymentExceedsOustandingCreditFacilityAmount(
                    amount,
                    outstanding.total(),
                ),
            );
        }

        let amounts = outstanding.allocate_payment(amount)?;

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
            disbursement_amount: repayment.amounts.disbursement,
            interest_amount: repayment.amounts.interest,
            audit_info,
            recorded_at,
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_info);
    }

    fn count_recorded_payments(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, CreditFacilityEvent::PaymentRecorded { .. }))
            .count()
    }

    pub fn last_collateralization_state(&self) -> CollateralizationState {
        if self.status() == CreditFacilityStatus::Closed {
            return CollateralizationState::NoCollateral;
        }

        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                CreditFacilityEvent::CollateralizationChanged { state, .. } => Some(*state),
                _ => None,
            })
            .unwrap_or(CollateralizationState::NoCollateral)
    }

    pub fn maybe_update_collateralization(
        &mut self,
        price: PriceOfOneBTC,
        upgrade_buffer_cvl_pct: CVLPct,
        audit_info: AuditInfo,
    ) -> Option<CollateralizationState> {
        let facility_cvl = self.facility_cvl(price);
        let last_collateralization_state = self.last_collateralization_state();

        let collateralization_update = match self.status() {
            CreditFacilityStatus::New => facility_cvl.total.collateralization_update(
                self.terms,
                last_collateralization_state,
                None,
                true,
            ),
            CreditFacilityStatus::Active => {
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
                    audit_info,
                });

            return Some(calculated_collateralization);
        }

        None
    }

    fn count_collateral_adjustments(&self) -> usize {
        self.events
            .iter()
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
            audit_info,
        });

        self.maybe_update_collateralization(price, upgrade_buffer_cvl_pct, audit_info)
    }
}

impl TryFrom<EntityEvents<CreditFacilityEvent>> for CreditFacility {
    type Error = EntityError;

    fn try_from(events: EntityEvents<CreditFacilityEvent>) -> Result<Self, Self::Error> {
        let mut builder = CreditFacilityBuilder::default();
        let mut terms = None;
        for event in events.iter() {
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
                CreditFacilityEvent::Approved { recorded_at, .. } => {
                    builder = builder.approved_at(*recorded_at).expires_at(
                        terms
                            .expect("terms should be set")
                            .duration
                            .expiration_date(*recorded_at),
                    )
                }
                CreditFacilityEvent::ApprovalAdded { .. } => (),
                CreditFacilityEvent::DisbursementInitiated { .. } => (),
                CreditFacilityEvent::DisbursementConcluded { .. } => (),
                CreditFacilityEvent::CollateralUpdated { .. } => (),
                CreditFacilityEvent::CollateralizationChanged { .. } => (),
                CreditFacilityEvent::PaymentRecorded { .. } => (),
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
    pub(super) customer_id: CustomerId,
    terms: TermValues,
    facility: UsdCents,
    account_ids: CreditFacilityAccountIds,
    customer_account_ids: CustomerLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewCreditFacility {
    pub fn builder() -> NewCreditFacilityBuilder {
        NewCreditFacilityBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<CreditFacilityEvent> {
        EntityEvents::init(
            self.id,
            [CreditFacilityEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
                customer_id: self.customer_id,
                terms: self.terms,
                facility: self.facility,
                account_ids: self.account_ids,
                customer_account_ids: self.customer_account_ids,
            }],
        )
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::{
        credit_facility::*,
        terms::{Duration, InterestInterval},
    };

    use super::*;

    fn default_terms() -> TermValues {
        TermValues::builder()
            .annual_rate(dec!(12))
            .duration(Duration::Months(3))
            .interval(InterestInterval::EndOfMonth)
            .liquidation_cvl(dec!(105))
            .margin_call_cvl(dec!(125))
            .initial_cvl(dec!(140))
            .build()
            .expect("should build a valid term")
    }

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: Subject::from(UserId::new()),
        }
    }

    fn default_price() -> PriceOfOneBTC {
        PriceOfOneBTC::new(UsdCents::from(5000000))
    }

    fn default_upgrade_buffer_cvl_pct() -> CVLPct {
        CVLPct::new(5)
    }

    fn facility_from(events: &Vec<CreditFacilityEvent>) -> CreditFacility {
        CreditFacility::try_from(EntityEvents::init(CreditFacilityId::new(), events.clone()))
            .unwrap()
    }

    fn initial_events() -> Vec<CreditFacilityEvent> {
        vec![CreditFacilityEvent::Initialized {
            id: CreditFacilityId::new(),
            audit_info: dummy_audit_info(),
            customer_id: CustomerId::new(),
            facility: UsdCents::from(100),
            terms: default_terms(),
            account_ids: CreditFacilityAccountIds::new(),
            customer_account_ids: CustomerLedgerAccountIds::new(),
        }]
    }

    #[test]
    fn is_disbursement_in_progress() {
        let mut events = initial_events();

        let first_idx = DisbursementIdx::FIRST;
        events.push(CreditFacilityEvent::DisbursementInitiated {
            idx: first_idx,
            amount: UsdCents::ONE,
            audit_info: dummy_audit_info(),
        });
        assert!(matches!(
            facility_from(&events).initiate_disbursement(dummy_audit_info(), UsdCents::ONE),
            Err(CreditFacilityError::DisbursementInProgress)
        ));

        events.push(CreditFacilityEvent::DisbursementConcluded {
            idx: first_idx,
            tx_id: LedgerTxId::new(),
            recorded_at: Utc::now(),
            audit_info: dummy_audit_info(),
        });
        assert!(facility_from(&events)
            .initiate_disbursement(dummy_audit_info(), UsdCents::ONE)
            .is_ok());
    }

    #[test]
    fn outstanding() {
        let mut events = initial_events();
        events.extend([
            CreditFacilityEvent::DisbursementInitiated {
                idx: DisbursementIdx::FIRST,
                amount: UsdCents::from(100),
                audit_info: dummy_audit_info(),
            },
            CreditFacilityEvent::DisbursementConcluded {
                idx: DisbursementIdx::FIRST,
                tx_id: LedgerTxId::new(),
                recorded_at: Utc::now(),
                audit_info: dummy_audit_info(),
            },
        ]);
        let credit_facility = facility_from(&events);

        assert_eq!(
            credit_facility.outstanding(),
            CreditFacilityReceivable {
                disbursed: UsdCents::from(100),
                interest: UsdCents::ZERO
            }
        );
    }

    #[test]
    fn collateral() {
        let mut credit_facility = facility_from(&initial_events());
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

    mod approve {
        use super::*;

        fn bank_manager_role() -> HashSet<Role> {
            let mut roles = HashSet::new();
            roles.insert(Role::BankManager);
            roles
        }

        fn admin_role() -> HashSet<Role> {
            let mut roles = HashSet::new();
            roles.insert(Role::Admin);
            roles
        }

        fn add_approvals(credit_facility: &mut CreditFacility) -> CreditFacilityApprovalData {
            let first_approval = credit_facility.add_approval(
                UserId::new(),
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );
            assert!(first_approval.is_ok());

            let second_approval = credit_facility.add_approval(
                UserId::new(),
                admin_role(),
                dummy_audit_info(),
                default_price(),
            );
            assert!(second_approval.is_ok());

            second_approval
                .unwrap()
                .expect("should return a credit facility approval")
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
        fn prevent_double_approve() {
            let mut credit_facility = facility_from(&initial_events());
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
            let credit_facility_approval = add_approvals(&mut credit_facility);
            credit_facility.confirm_approval(
                credit_facility_approval,
                Utc::now(),
                dummy_audit_info(),
            );

            let third_approval = credit_facility.add_approval(
                UserId::new(),
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );
            assert!(matches!(
                third_approval,
                Err(CreditFacilityError::AlreadyApproved)
            ));
        }

        #[test]
        fn check_approved_at() {
            let mut credit_facility = facility_from(&initial_events());
            assert_eq!(credit_facility.approved_at, None);
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

            let credit_facility_approval = add_approvals(&mut credit_facility);
            credit_facility.confirm_approval(
                credit_facility_approval,
                approval_time,
                dummy_audit_info(),
            );
            assert_eq!(credit_facility.approved_at, Some(approval_time));
            assert!(credit_facility.expires_at.is_some())
        }

        #[test]
        fn cannot_approve_if_credit_facility_has_no_collateral() {
            let mut credit_facility = facility_from(&initial_events());
            let res = credit_facility.add_approval(
                UserId::new(),
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );
            assert!(matches!(res, Err(CreditFacilityError::NoCollateral)));
        }

        #[test]
        fn reject_credit_facility_approval_below_margin_limit() {
            let mut credit_facility = facility_from(&initial_events());

            let credit_facility_collateral_update = credit_facility
                .initiate_collateral_update(Satoshis::from(100))
                .unwrap();
            credit_facility.confirm_collateral_update(
                credit_facility_collateral_update,
                Utc::now(),
                dummy_audit_info(),
                default_price(),
                default_upgrade_buffer_cvl_pct(),
            );

            let first_approval = credit_facility.add_approval(
                UserId::new(),
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );
            assert!(matches!(
                first_approval,
                Err(CreditFacilityError::BelowMarginLimit)
            ));
        }

        #[test]
        fn two_admins_can_approve() {
            let mut credit_facility = facility_from(&initial_events());
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
            let _first_admin_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    admin_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            let _second_admin_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    admin_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            assert!(credit_facility.approval_threshold_met());
        }

        #[test]
        fn admin_and_bank_manager_can_approve() {
            let mut credit_facility = facility_from(&initial_events());
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
            let _admin_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    admin_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            let _bank_manager_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    bank_manager_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            assert!(credit_facility.approval_threshold_met());
        }

        #[test]
        fn user_with_both_admin_and_bank_manager_role_cannot_approve() {
            let mut credit_facility = facility_from(&initial_events());
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
            let admin_and_bank_manager =
                admin_role().union(&bank_manager_role()).cloned().collect();
            let _approval = credit_facility
                .add_approval(
                    UserId::new(),
                    admin_and_bank_manager,
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            assert!(!credit_facility.approval_threshold_met());
        }

        #[test]
        fn two_bank_managers_cannot_approve() {
            let mut credit_facility = facility_from(&initial_events());
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
            let _first_bank_manager_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    bank_manager_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();
            let _second_bank_manager_approval = credit_facility
                .add_approval(
                    UserId::new(),
                    bank_manager_role(),
                    dummy_audit_info(),
                    default_price(),
                )
                .unwrap();

            assert!(!credit_facility.approval_threshold_met());
        }

        #[test]
        fn same_user_cannot_approve_twice() {
            let mut credit_facility = facility_from(&initial_events());
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

            let user_id = UserId::new();

            let first_approval = credit_facility.add_approval(
                user_id,
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );

            assert!(first_approval.is_ok());

            let second_approval = credit_facility.add_approval(
                user_id,
                bank_manager_role(),
                dummy_audit_info(),
                default_price(),
            );

            assert!(matches!(
                second_approval,
                Err(CreditFacilityError::UserCannotApproveTwice)
            ));
        }

        #[test]
        fn status() {
            let mut credit_facility = facility_from(&initial_events());
            assert_eq!(credit_facility.status(), CreditFacilityStatus::New);

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
            let credit_facility_approval = add_approvals(&mut credit_facility);
            credit_facility.confirm_approval(
                credit_facility_approval,
                Utc::now(),
                dummy_audit_info(),
            );
            assert_eq!(credit_facility.status(), CreditFacilityStatus::Active);
        }
    }
}
