use async_graphql::*;

use crate::collateral::CollateralBase;
use crate::primitives::*;
pub use lana_app::credit::Liquidation as DomainLiquidation;

#[derive(SimpleObject, Clone)]
#[graphql(name = "Liquidation", complex)]
pub struct LiquidationBase {
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

impl From<DomainLiquidation> for LiquidationBase {
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
impl LiquidationBase {
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

    async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBase> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let collaterals: std::collections::HashMap<_, CollateralBase> = app
            .credit()
            .collaterals()
            .find_all(&[self.entity.collateral_id])
            .await?;
        Ok(collaterals
            .into_values()
            .next()
            .expect("Collateral not found"))
    }
}
