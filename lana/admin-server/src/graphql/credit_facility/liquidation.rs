use async_graphql::{connection::*, *};
use es_entity::Sort;

use crate::{
    graphql::{
        event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
        loader::LanaDataLoader,
    },
    primitives::*,
};
pub use lana_app::credit::{
    Liquidation as DomainLiquidation, LiquidationsSortBy as DomainLiquidationsSortBy,
};

use super::{Collateral, SortDirection};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Liquidation {
    id: ID,
    liquidation_id: UUID,
    collateral_id: UUID,
    expected_to_receive: UsdCents,
    sent_total: Satoshis,
    amount_received: UsdCents,
    created_at: Timestamp,
    completed: bool,

    #[graphql(skip)]
    pub entity: Arc<DomainLiquidation>,
}

impl From<DomainLiquidation> for Liquidation {
    fn from(liquidation: DomainLiquidation) -> Self {
        Self {
            id: liquidation.id.to_global_id(),
            liquidation_id: UUID::from(liquidation.id),
            collateral_id: UUID::from(liquidation.collateral_id),
            expected_to_receive: liquidation.expected_to_receive,
            sent_total: liquidation.sent_total,
            amount_received: liquidation.amount_received,
            created_at: liquidation.created_at().into(),
            completed: liquidation.is_completed(),
            entity: Arc::new(liquidation),
        }
    }
}

#[derive(SimpleObject)]
pub struct LiquidationCollateralSent {
    amount: Satoshis,
    ledger_tx_id: UUID,
}

#[derive(SimpleObject)]
pub struct LiquidationProceedsReceived {
    amount: UsdCents,
    ledger_tx_id: UUID,
}

#[ComplexObject]
impl Liquidation {
    async fn sent_collateral(&self) -> Vec<LiquidationCollateralSent> {
        self.entity
            .collateral_sent_out()
            .into_iter()
            .map(|(amount, ledger_tx_id)| LiquidationCollateralSent {
                amount,
                ledger_tx_id: ledger_tx_id.into(),
            })
            .collect()
    }

    async fn received_proceeds(&self) -> Vec<LiquidationProceedsReceived> {
        self.entity
            .proceeds_received()
            .into_iter()
            .map(|(amount, ledger_tx_id)| LiquidationProceedsReceived {
                amount,
                ledger_tx_id: ledger_tx_id.into(),
            })
            .collect()
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

    async fn collateral(&self, ctx: &Context<'_>) -> Result<Collateral> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("Collateral not found");
        Ok(collateral)
    }
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiquidationsSortBy {
    #[default]
    CreatedAt,
    ExpectedToReceive,
    AmountReceived,
    SentTotal,
}

impl From<LiquidationsSortBy> for DomainLiquidationsSortBy {
    fn from(by: LiquidationsSortBy) -> Self {
        match by {
            LiquidationsSortBy::CreatedAt => DomainLiquidationsSortBy::CreatedAt,
            LiquidationsSortBy::ExpectedToReceive => DomainLiquidationsSortBy::ExpectedToReceive,
            LiquidationsSortBy::AmountReceived => DomainLiquidationsSortBy::AmountReceived,
            LiquidationsSortBy::SentTotal => DomainLiquidationsSortBy::SentTotal,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct LiquidationsSort {
    #[graphql(default)]
    pub by: LiquidationsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<LiquidationsSort> for Sort<DomainLiquidationsSortBy> {
    fn from(sort: LiquidationsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<LiquidationsSort> for DomainLiquidationsSortBy {
    fn from(sort: LiquidationsSort) -> Self {
        sort.by.into()
    }
}
