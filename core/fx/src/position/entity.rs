use derive_builder::Builder;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::FxPositionId;

use super::error::FxPositionError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "FxPositionId")]
pub enum FxPositionEvent {
    Initialized {
        id: FxPositionId,
        currency: String,
    },
    PositionIncreased {
        foreign_amount: Decimal,
        functional_cost: Decimal,
    },
    PositionDecreased {
        foreign_amount: Decimal,
        functional_proceeds: Decimal,
        realized_gain_loss: Decimal,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityHydrationError"))]
pub struct FxPosition {
    pub id: FxPositionId,
    pub currency: String,
    pub(super) balance: Decimal,
    pub(super) cost_basis: Decimal,
    events: EntityEvents<FxPositionEvent>,
}

impl FxPosition {
    pub fn balance(&self) -> Decimal {
        self.balance
    }

    pub fn cost_basis(&self) -> Decimal {
        self.cost_basis
    }

    /// Weighted average cost per unit of foreign currency.
    /// Returns None if the position balance is zero.
    pub fn weighted_average_cost(&self) -> Option<Decimal> {
        if self.balance == Decimal::ZERO {
            None
        } else {
            Some(self.cost_basis / self.balance)
        }
    }

    /// Increase the position when foreign currency is acquired.
    /// `foreign_amount` is the amount of foreign currency added.
    /// `functional_cost` is how much functional currency was spent to acquire it.
    pub(crate) fn increase_position(
        &mut self,
        foreign_amount: Decimal,
        functional_cost: Decimal,
    ) -> Result<(), FxPositionError> {
        if foreign_amount <= Decimal::ZERO {
            return Err(FxPositionError::InvalidAmount);
        }
        self.events.push(FxPositionEvent::PositionIncreased {
            foreign_amount,
            functional_cost,
        });
        self.balance += foreign_amount;
        self.cost_basis += functional_cost;
        Ok(())
    }

    /// Decrease the position when foreign currency is disposed of.
    /// Returns the realized gain/loss (positive = gain, negative = loss).
    pub(crate) fn decrease_position(
        &mut self,
        foreign_amount: Decimal,
        functional_proceeds: Decimal,
    ) -> Result<Decimal, FxPositionError> {
        if foreign_amount <= Decimal::ZERO {
            return Err(FxPositionError::InvalidAmount);
        }
        if foreign_amount > self.balance {
            return Err(FxPositionError::InsufficientBalance);
        }

        let wac = self
            .weighted_average_cost()
            .expect("balance is non-zero since we checked above");
        let cost_of_disposed = wac * foreign_amount;
        let realized_gain_loss = functional_proceeds - cost_of_disposed;

        self.events.push(FxPositionEvent::PositionDecreased {
            foreign_amount,
            functional_proceeds,
            realized_gain_loss,
        });
        self.balance -= foreign_amount;
        self.cost_basis -= cost_of_disposed;
        Ok(realized_gain_loss)
    }
}

impl TryFromEvents<FxPositionEvent> for FxPosition {
    fn try_from_events(
        events: EntityEvents<FxPositionEvent>,
    ) -> Result<Self, EntityHydrationError> {
        let mut builder = FxPositionBuilder::default();
        let mut balance = Decimal::ZERO;
        let mut cost_basis = Decimal::ZERO;
        for event in events.iter_all() {
            match event {
                FxPositionEvent::Initialized { id, currency, .. } => {
                    builder = builder.id(*id).currency(currency.clone());
                }
                FxPositionEvent::PositionIncreased {
                    foreign_amount,
                    functional_cost,
                    ..
                } => {
                    balance += foreign_amount;
                    cost_basis += functional_cost;
                }
                FxPositionEvent::PositionDecreased { foreign_amount, .. } => {
                    let wac = if balance != Decimal::ZERO {
                        cost_basis / balance
                    } else {
                        Decimal::ZERO
                    };
                    let cost_of_disposed = wac * foreign_amount;
                    balance -= foreign_amount;
                    cost_basis -= cost_of_disposed;
                }
            }
        }
        builder
            .balance(balance)
            .cost_basis(cost_basis)
            .events(events)
            .build()
    }
}

#[derive(Debug, Builder)]
pub struct NewFxPosition {
    #[builder(setter(into))]
    pub(super) id: FxPositionId,
    pub(super) currency: String,
}

impl NewFxPosition {
    pub fn builder() -> NewFxPositionBuilder {
        NewFxPositionBuilder::default()
    }
}

impl IntoEvents<FxPositionEvent> for NewFxPosition {
    fn into_events(self) -> EntityEvents<FxPositionEvent> {
        EntityEvents::init(
            self.id,
            [FxPositionEvent::Initialized {
                id: self.id,
                currency: self.currency,
            }],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn new_position(currency: &str) -> FxPosition {
        let id = FxPositionId::new();
        let new = NewFxPosition::builder()
            .id(id)
            .currency(currency.to_string())
            .build()
            .unwrap();
        let events = new.into_events();
        FxPosition::try_from_events(events).unwrap()
    }

    #[test]
    fn test_initial_state() {
        let pos = new_position("EUR");
        assert_eq!(pos.balance(), dec!(0));
        assert_eq!(pos.cost_basis(), dec!(0));
        assert!(pos.weighted_average_cost().is_none());
    }

    #[test]
    fn test_increase_position() {
        let mut pos = new_position("EUR");
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        assert_eq!(pos.balance(), dec!(100));
        assert_eq!(pos.cost_basis(), dec!(110));
        assert_eq!(pos.weighted_average_cost(), Some(dec!(1.1)));
    }

    #[test]
    fn test_wac_with_multiple_increases() {
        let mut pos = new_position("EUR");
        // Buy 100 EUR at 1.10 USD/EUR
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        // Buy 200 EUR at 1.20 USD/EUR
        pos.increase_position(dec!(200), dec!(240)).unwrap();
        // WAC = 350 / 300 = 1.1666...
        assert_eq!(pos.balance(), dec!(300));
        assert_eq!(pos.cost_basis(), dec!(350));
        let wac = pos.weighted_average_cost().unwrap();
        // 350/300 = 1.16666...
        assert!(wac > dec!(1.166) && wac < dec!(1.167));
    }

    #[test]
    fn test_decrease_with_gain() {
        let mut pos = new_position("EUR");
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        // WAC = 1.10. Sell 50 EUR for 60 USD (rate 1.20)
        // Cost = 50 * 1.10 = 55. Proceeds = 60. G/L = +5
        let gl = pos.decrease_position(dec!(50), dec!(60)).unwrap();
        assert_eq!(gl, dec!(5));
        assert_eq!(pos.balance(), dec!(50));
        assert_eq!(pos.cost_basis(), dec!(55));
    }

    #[test]
    fn test_decrease_with_loss() {
        let mut pos = new_position("EUR");
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        // WAC = 1.10. Sell 50 EUR for 50 USD (rate 1.00)
        // Cost = 50 * 1.10 = 55. Proceeds = 50. G/L = -5
        let gl = pos.decrease_position(dec!(50), dec!(50)).unwrap();
        assert_eq!(gl, dec!(-5));
    }

    #[test]
    fn test_decrease_insufficient_balance() {
        let mut pos = new_position("EUR");
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        let err = pos.decrease_position(dec!(150), dec!(165)).unwrap_err();
        assert!(matches!(err, FxPositionError::InsufficientBalance));
    }

    #[test]
    fn test_hydration_roundtrip() {
        let mut pos = new_position("EUR");
        pos.increase_position(dec!(100), dec!(110)).unwrap();
        pos.increase_position(dec!(200), dec!(240)).unwrap();
        pos.decrease_position(dec!(50), dec!(60)).unwrap();

        // Rebuild from events
        let events = pos.events;
        let rebuilt = FxPosition::try_from_events(events).unwrap();
        assert_eq!(rebuilt.balance(), dec!(250));
        // Cost basis after decrease: 350 - (350/300)*50 = 350 - 58.333... = 291.666...
        assert_eq!(rebuilt.cost_basis(), pos.cost_basis);
    }
}
