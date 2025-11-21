use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use core_credit::CreditFacilityId;
use core_customer::CustomerId;
use core_deposit::DepositAccountId;

use crate::primitives::*;

use super::error::FundingLinkError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "FundingLinkId")]
pub enum FundingLinkEvent {
    Initialized {
        id: FundingLinkId,
        customer_id: CustomerId,
        deposit_account_id: DepositAccountId,
        credit_facility_id: CreditFacilityId,
    },
    Activated,
    Deactivated,
    Broken {
        reason: BrokenReason,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct FundingLink {
    pub id: FundingLinkId,
    pub customer_id: CustomerId,
    pub deposit_account_id: DepositAccountId,
    pub credit_facility_id: CreditFacilityId,
    pub status: LinkStatus,

    events: EntityEvents<FundingLinkEvent>,
}

impl FundingLink {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn activate(&mut self) -> Idempotent<()> {
        if self.status == LinkStatus::Active {
            return Idempotent::Ignored;
        }

        self.events.push(FundingLinkEvent::Activated);
        self.status = LinkStatus::Active;

        Idempotent::Executed(())
    }

    pub fn deactivate(&mut self) -> Idempotent<()> {
        if self.status == LinkStatus::Inactive {
            return Idempotent::Ignored;
        }

        self.events.push(FundingLinkEvent::Deactivated);
        self.status = LinkStatus::Inactive;

        Idempotent::Executed(())
    }

    pub fn mark_broken(
        &mut self,
        reason: BrokenReason,
    ) -> Result<Idempotent<()>, FundingLinkError> {
        if self.status == LinkStatus::Broken {
            return Err(FundingLinkError::LinkAlreadyBroken);
        }

        self.events.push(FundingLinkEvent::Broken { reason });
        self.status = LinkStatus::Broken;

        Ok(Idempotent::Executed(()))
    }

    pub fn is_active(&self) -> bool {
        self.status == LinkStatus::Active
    }
}

#[derive(Debug, Builder)]
pub struct NewFundingLink {
    #[builder(setter(into))]
    pub(super) id: FundingLinkId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    #[builder(setter(into))]
    pub(super) deposit_account_id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) credit_facility_id: CreditFacilityId,
}

impl NewFundingLink {
    pub fn builder() -> NewFundingLinkBuilder {
        NewFundingLinkBuilder::default()
    }
}

impl TryFromEvents<FundingLinkEvent> for FundingLink {
    fn try_from_events(events: EntityEvents<FundingLinkEvent>) -> Result<Self, EsEntityError> {
        let mut builder = FundingLinkBuilder::default();
        let mut status = LinkStatus::Inactive;

        for event in events.iter_all() {
            match event {
                FundingLinkEvent::Initialized {
                    id,
                    customer_id,
                    deposit_account_id,
                    credit_facility_id,
                } => {
                    builder = builder
                        .id(*id)
                        .customer_id(*customer_id)
                        .deposit_account_id(*deposit_account_id)
                        .credit_facility_id(*credit_facility_id);
                }
                FundingLinkEvent::Activated => {
                    status = LinkStatus::Active;
                }
                FundingLinkEvent::Deactivated => {
                    status = LinkStatus::Inactive;
                }
                FundingLinkEvent::Broken { .. } => {
                    status = LinkStatus::Broken;
                }
            }
        }

        builder.status(status).events(events).build()
    }
}

impl es_entity::IntoEvents<FundingLinkEvent> for NewFundingLink {
    fn into_events(self) -> EntityEvents<FundingLinkEvent> {
        EntityEvents::init(
            self.id,
            [FundingLinkEvent::Initialized {
                id: self.id,
                customer_id: self.customer_id,
                deposit_account_id: self.deposit_account_id,
                credit_facility_id: self.credit_facility_id,
            }],
        )
    }
}
