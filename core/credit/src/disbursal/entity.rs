use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::{ledger::DisbursalLedgerAccountIds, primitives::*};

use core_credit_collection::NewObligation;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DisbursalId")]
pub enum DisbursalEvent {
    Initialized {
        id: DisbursalId,
        ledger_tx_id: LedgerTxId,
        approval_process_id: ApprovalProcessId,
        facility_id: CreditFacilityId,
        amount: UsdCents,
        account_ids: DisbursalLedgerAccountIds,
        disbursal_credit_account_id: CalaAccountId,
        due_date: EffectiveDate,
        overdue_date: Option<EffectiveDate>,
        liquidation_date: Option<EffectiveDate>,
        public_id: PublicId,
    },
    ApprovalProcessConcluded {
        approval_process_id: ApprovalProcessId,
        approved: bool,
    },
    Settled {
        ledger_tx_id: LedgerTxId,
        obligation_id: ObligationId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
    },
    Cancelled {
        ledger_tx_id: LedgerTxId,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Disbursal {
    pub id: DisbursalId,
    pub approval_process_id: ApprovalProcessId,
    pub facility_id: CreditFacilityId,
    pub amount: UsdCents,
    pub account_ids: DisbursalLedgerAccountIds,
    pub disbursal_credit_account_id: CalaAccountId,
    pub due_date: EffectiveDate,
    pub overdue_date: Option<EffectiveDate>,
    pub liquidation_date: Option<EffectiveDate>,
    pub initiated_tx_id: LedgerTxId,
    #[builder(setter(strip_option), default)]
    pub concluded_tx_id: Option<LedgerTxId>,
    pub public_id: PublicId,
    events: EntityEvents<DisbursalEvent>,
}

impl TryFromEvents<DisbursalEvent> for Disbursal {
    fn try_from_events(events: EntityEvents<DisbursalEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DisbursalBuilder::default();
        for event in events.iter_all() {
            match event {
                DisbursalEvent::Initialized {
                    id,
                    ledger_tx_id,
                    approval_process_id,
                    facility_id,
                    amount,
                    account_ids,
                    disbursal_credit_account_id,
                    due_date,
                    overdue_date,
                    liquidation_date,
                    public_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .initiated_tx_id(*ledger_tx_id)
                        .approval_process_id(*approval_process_id)
                        .facility_id(*facility_id)
                        .amount(*amount)
                        .account_ids(*account_ids)
                        .disbursal_credit_account_id(*disbursal_credit_account_id)
                        .due_date(*due_date)
                        .overdue_date(*overdue_date)
                        .liquidation_date(*liquidation_date)
                        .public_id(public_id.clone())
                }
                DisbursalEvent::Settled { ledger_tx_id, .. } => {
                    builder = builder.concluded_tx_id(*ledger_tx_id)
                }
                DisbursalEvent::Cancelled { ledger_tx_id, .. } => {
                    builder = builder.concluded_tx_id(*ledger_tx_id)
                }
                DisbursalEvent::ApprovalProcessConcluded { .. } => (),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DisbursalSettlement {
    pub tx_id: LedgerTxId,
    pub effective: chrono::NaiveDate,
}

impl Disbursal {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn is_approval_process_concluded(&self) -> bool {
        self.events
            .iter_all()
            .any(|e| matches!(e, DisbursalEvent::ApprovalProcessConcluded { .. }))
    }
    pub fn status(&self) -> DisbursalStatus {
        if self.is_confirmed() {
            DisbursalStatus::Confirmed
        } else {
            match self.is_approved() {
                Some(true) => DisbursalStatus::Approved,
                Some(false) => DisbursalStatus::Denied,
                None => DisbursalStatus::New,
            }
        }
    }

    pub fn settlement(&self) -> Option<DisbursalSettlement> {
        self.events.iter_all().find_map(|event| match event {
            DisbursalEvent::Settled {
                ledger_tx_id,
                effective,
                ..
            } => Some(DisbursalSettlement {
                tx_id: *ledger_tx_id,
                effective: *effective,
            }),
            _ => None,
        })
    }

    pub fn obligation_id(&self) -> Option<ObligationId> {
        self.events.iter_all().find_map(|event| match event {
            DisbursalEvent::Settled { obligation_id, .. } => Some(*obligation_id),
            _ => None,
        })
    }

    pub(crate) fn approval_process_concluded_for_initial_disbursal(
        &mut self,
        effective: chrono::NaiveDate,
    ) -> Idempotent<Option<NewObligation>> {
        idempotency_guard!(
            self.events.iter_all(),
            DisbursalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: self.approval_process_id,
            approved: true,
        });
        let tx_id = self.initiated_tx_id;
        let tx_ref: &str = &format!("initial-disbursal-{}", self.id);
        let new_obligation = if let Idempotent::Executed(new_obligation) =
            self.settle_disbursal(tx_id, tx_ref, effective)
        {
            Some(new_obligation)
        } else {
            return Idempotent::AlreadyApplied;
        };

        self.concluded_tx_id = Some(tx_id);

        Idempotent::Executed(new_obligation)
    }

    pub(crate) fn approval_process_concluded(
        &mut self,
        approved: bool,
        effective: chrono::NaiveDate,
    ) -> Idempotent<Option<NewObligation>> {
        idempotency_guard!(
            self.events.iter_all(),
            DisbursalEvent::ApprovalProcessConcluded { .. }
        );
        self.events.push(DisbursalEvent::ApprovalProcessConcluded {
            approval_process_id: self.approval_process_id,
            approved,
        });
        let tx_id = LedgerTxId::new();
        let tx_ref: &str = &format!("disbursal-{}", self.id);
        let new_obligation = if approved {
            if let Idempotent::Executed(new_obligation) =
                self.settle_disbursal(tx_id, tx_ref, effective)
            {
                Some(new_obligation)
            } else {
                return Idempotent::AlreadyApplied;
            }
        } else {
            self.events.push(DisbursalEvent::Cancelled {
                ledger_tx_id: tx_id,
            });
            None
        };
        self.concluded_tx_id = Some(tx_id);

        Idempotent::Executed(new_obligation)
    }

    pub(super) fn is_approved(&self) -> Option<bool> {
        for event in self.events.iter_all() {
            if let DisbursalEvent::ApprovalProcessConcluded { approved, .. } = event {
                return Some(*approved);
            }
        }
        None
    }

    fn settle_disbursal(
        &mut self,
        tx_id: LedgerTxId,
        tx_ref: &str,
        effective: chrono::NaiveDate,
    ) -> Idempotent<NewObligation> {
        idempotency_guard!(self.events.iter_all(), DisbursalEvent::Settled { .. });
        let obligation_id = ObligationId::new();
        self.events.push(DisbursalEvent::Settled {
            ledger_tx_id: tx_id,
            obligation_id,
            amount: self.amount,
            effective,
        });

        Idempotent::Executed(
            NewObligation::builder()
                .id(obligation_id)
                .beneficiary_id(self.facility_id)
                .obligation_type(ObligationType::Disbursal)
                .reference(tx_ref.to_string())
                .amount(self.amount)
                .tx_id(tx_id)
                .receivable_account_ids(self.account_ids.into())
                .defaulted_account_id(self.account_ids.defaulted_account_id())
                .due_date(self.due_date)
                .overdue_date(self.overdue_date)
                .liquidation_date(self.liquidation_date)
                .effective(effective)
                .build()
                .expect("could not build new disbursal obligation"),
        )
    }

    pub(super) fn is_confirmed(&self) -> bool {
        for event in self.events.iter_all() {
            match event {
                DisbursalEvent::Settled { .. } => return true,
                _ => continue,
            }
        }
        false
    }

    pub fn ledger_tx_ids(&self) -> Vec<LedgerTxId> {
        self.events
            .iter_all()
            .filter_map(|e| match e {
                DisbursalEvent::Initialized { id, .. } => Some((*id).into()),
                DisbursalEvent::Settled { ledger_tx_id, .. } => Some(*ledger_tx_id),
                DisbursalEvent::Cancelled { ledger_tx_id, .. } => Some(*ledger_tx_id),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct NewDisbursal {
    #[builder(setter(into))]
    pub(super) id: DisbursalId,
    #[builder(setter(into))]
    pub(crate) approval_process_id: ApprovalProcessId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
    pub(super) amount: UsdCents,
    pub(super) account_ids: DisbursalLedgerAccountIds,
    #[builder(setter(into))]
    pub(super) disbursal_credit_account_id: CalaAccountId,
    pub(super) due_date: EffectiveDate,
    pub(super) overdue_date: Option<EffectiveDate>,
    pub(super) liquidation_date: Option<EffectiveDate>,
    #[builder(setter(into))]
    pub(super) public_id: PublicId,
}

impl NewDisbursalBuilder {
    pub fn unwrap_id(&self) -> DisbursalId {
        self.id.expect("disbursal_id not set")
    }
}

impl NewDisbursalBuilder {
    fn validate(&self) -> Result<(), String> {
        match self.amount {
            Some(amount) if amount.is_zero() => Err("Disbursal amount cannot be zero".to_string()),
            _ => Ok(()),
        }
    }
}

impl NewDisbursal {
    pub fn builder() -> NewDisbursalBuilder {
        NewDisbursalBuilder::default()
    }
}

impl IntoEvents<DisbursalEvent> for NewDisbursal {
    fn into_events(self) -> EntityEvents<DisbursalEvent> {
        EntityEvents::init(
            self.id,
            [DisbursalEvent::Initialized {
                id: self.id,
                ledger_tx_id: self.id.into(),
                approval_process_id: self.approval_process_id,
                facility_id: self.credit_facility_id,
                amount: self.amount,
                account_ids: self.account_ids,
                disbursal_credit_account_id: self.disbursal_credit_account_id,
                due_date: self.due_date,
                overdue_date: self.overdue_date,
                liquidation_date: self.liquidation_date,
                public_id: self.public_id,
            }],
        )
    }
}
