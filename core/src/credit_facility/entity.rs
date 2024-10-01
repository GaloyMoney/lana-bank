use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    entity::*,
    ledger::{
        credit_facility::{CreditFacilityAccountIds, CreditFacilityApprovalData},
        customer::CustomerLedgerAccountIds,
    },
    primitives::*,
    terms::{CVLPct, TermValues},
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
}

impl EntityEvent for CreditFacilityEvent {
    type EntityId = CreditFacilityId;
    fn event_table_name() -> &'static str {
        "credit_facility_events"
    }
}

pub struct FacilityReceivable {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl FacilityReceivable {
    pub fn total(&self) -> UsdCents {
        self.interest + self.disbursed
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
    ) -> Result<Option<CreditFacilityApprovalData>, CreditFacilityError> {
        if self.has_user_previously_approved(approving_user_id) {
            return Err(CreditFacilityError::UserCannotApproveTwice);
        }

        if self.is_approved() {
            return Err(CreditFacilityError::AlreadyApproved);
        }

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

    pub fn outstanding(&self) -> FacilityReceivable {
        FacilityReceivable {
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

    pub fn cvl(&self, price: PriceOfOneBTC) -> FacilityCVL {
        let collateral_value = price.sats_to_cents_round_down(self.collateral());
        if collateral_value == UsdCents::ZERO {
            return FacilityCVL::ZERO;
        }

        FacilityCVL {
            total: CVLPct::from_loan_amounts(
                collateral_value,
                self.outstanding().total() + self.facility_remaining(),
            ),
            disbursed: CVLPct::from_loan_amounts(collateral_value, self.outstanding().total()),
        }
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
    ) {
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

    fn terms() -> TermValues {
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

    fn facility_from(events: &Vec<CreditFacilityEvent>) -> CreditFacility {
        CreditFacility::try_from(EntityEvents::init(CreditFacilityId::new(), events.clone()))
            .unwrap()
    }

    #[test]
    fn is_disbursement_in_progress() {
        let mut events = vec![CreditFacilityEvent::Initialized {
            id: CreditFacilityId::new(),
            audit_info: dummy_audit_info(),
            customer_id: CustomerId::new(),
            facility: UsdCents::from(10000),
            terms: terms(),
            account_ids: CreditFacilityAccountIds::new(),
            customer_account_ids: CustomerLedgerAccountIds::new(),
        }];

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
}
