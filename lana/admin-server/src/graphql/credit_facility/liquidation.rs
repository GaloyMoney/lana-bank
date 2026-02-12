use async_graphql::*;

use crate::{graphql::loader::LanaDataLoader, primitives::*};
pub(crate) use lana_app::credit::Liquidation as DomainLiquidation;

use super::Collateral;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(crate) struct Liquidation {
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
pub(crate) struct LiquidationCollateralSent {
    amount: Satoshis,
    ledger_tx_id: UUID,
}

#[derive(SimpleObject)]
pub(crate) struct LiquidationProceedsReceived {
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

    async fn collateral(&self, ctx: &Context<'_>) -> Result<Collateral> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("Collateral not found");
        Ok(collateral)
    }
}
