use async_graphql::{connection::*, *};

use crate::{
    graphql::{
        custody::Wallet,
        customer::*,
        event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
        loader::LanaDataLoader,
        terms::TermValues,
    },
    primitives::*,
};

use super::{
    ApprovalProcess, CollateralBalance, CreditFacilityRepaymentPlanEntry,
    PendingCreditFacilityCollateralizationUpdated, Sort, SortDirection,
};

pub use lana_app::credit::{
    PendingCreditFacilitiesCursor,
    PendingCreditFacilitiesFilters as DomainPendingCreditFacilitiesFilters,
    PendingCreditFacilitiesSortBy as DomainPendingCreditFacilitiesSortBy,
    PendingCreditFacility as DomainPendingCreditFacility,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PendingCreditFacility {
    id: ID,
    pending_credit_facility_id: UUID,
    /// Canonical credit facility identifier reserved for this facility.
    /// Today this matches `pendingCreditFacilityId`, but clients should use
    /// this field when they need the active facility reference.
    credit_facility_id: UUID,
    collateral_id: UUID,
    status: PendingCreditFacilityStatus,
    approval_process_id: UUID,
    created_at: Timestamp,
    collateralization_state: PendingCreditFacilityCollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainPendingCreditFacility>,
}

#[ComplexObject]
impl PendingCreditFacility {
    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Wallet> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("pending credit facility has collateral");

        let wallet_id = collateral
            .wallet_id
            .expect("pending credit facility has wallet");
        Ok(loader
            .load_one(WalletId::from(wallet_id))
            .await?
            .expect("wallet exists"))
    }

    async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let collateral = app
            .credit()
            .pending_credit_facilities()
            .collateral(sub, self.entity.id)
            .await?;

        Ok(CollateralBalance {
            btc_balance: collateral,
        })
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn credit_facility_terms(&self) -> TermValues {
        self.entity.terms.into()
    }

    async fn repayment_plan(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityRepaymentPlanEntry>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .repayment_plans()
            .find_for_credit_facility_id(sub, self.entity.id)
            .await?)
    }

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }
}

#[derive(InputObject)]
pub struct PendingCreditFacilitiesFilter {
    pub status: Option<PendingCreditFacilityStatus>,
    pub collateralization_state: Option<PendingCreditFacilityCollateralizationState>,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PendingCreditFacilitiesSortBy {
    #[default]
    CreatedAt,
    FacilityAmount,
}

impl From<PendingCreditFacilitiesSortBy> for DomainPendingCreditFacilitiesSortBy {
    fn from(by: PendingCreditFacilitiesSortBy) -> Self {
        match by {
            PendingCreditFacilitiesSortBy::CreatedAt => {
                DomainPendingCreditFacilitiesSortBy::CreatedAt
            }
            PendingCreditFacilitiesSortBy::FacilityAmount => {
                DomainPendingCreditFacilitiesSortBy::Amount
            }
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct PendingCreditFacilitiesSort {
    #[graphql(default)]
    pub by: PendingCreditFacilitiesSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<PendingCreditFacilitiesSort> for Sort<DomainPendingCreditFacilitiesSortBy> {
    fn from(sort: PendingCreditFacilitiesSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<PendingCreditFacilitiesSort> for DomainPendingCreditFacilitiesSortBy {
    fn from(sort: PendingCreditFacilitiesSort) -> Self {
        sort.by.into()
    }
}

impl From<DomainPendingCreditFacility> for PendingCreditFacility {
    fn from(pending_credit_facility: DomainPendingCreditFacility) -> Self {
        let created_at = pending_credit_facility.created_at();

        Self {
            id: pending_credit_facility.id.to_global_id(),
            pending_credit_facility_id: UUID::from(pending_credit_facility.id),
            credit_facility_id: UUID::from(CreditFacilityId::from(pending_credit_facility.id)),
            collateral_id: UUID::from(pending_credit_facility.collateral_id),
            approval_process_id: UUID::from(pending_credit_facility.approval_process_id),
            created_at: created_at.into(),
            facility_amount: pending_credit_facility.amount,
            collateralization_state: pending_credit_facility.last_collateralization_state(),
            status: pending_credit_facility.status(),

            entity: Arc::new(pending_credit_facility),
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCollateralizationPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}

#[derive(SimpleObject)]
pub struct PendingCreditFacilityCompleted {
    pub status: PendingCreditFacilityStatus,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCompletedPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCompleted,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCompletedPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}
