use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use core_price::{CorePriceEvent, PRICE_UPDATED_EVENT_TYPE, Price, PriceOfOneBTC};
use obix::out::Outbox;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "module")]
pub enum DummyEvent {
    Price(CorePriceEvent),
    #[serde(other)]
    Unknown,
}

impl obix::out::OutboxEventMarker<CorePriceEvent> for DummyEvent {
    fn as_event(&self) -> Option<&CorePriceEvent> {
        match self {
            DummyEvent::Price(event) => Some(event),
            DummyEvent::Unknown => None,
        }
    }
}

impl From<CorePriceEvent> for DummyEvent {
    fn from(event: CorePriceEvent) -> Self {
        DummyEvent::Price(event)
    }
}

pub async fn wait_for_price_to_be_updated(
    price: &Price,
    expected_price: PriceOfOneBTC,
    attempts: u32,
) -> anyhow::Result<PriceOfOneBTC> {
    let mut current_price = price.usd_cents_per_btc().await;

    for attempt in 0..attempts {
        if current_price == expected_price {
            break;
        }

        if attempt + 1 == attempts {
            break;
        }

        sleep(Duration::from_millis(100)).await;
        current_price = price.usd_cents_per_btc().await;
    }

    Ok(current_price)
}

pub async fn publish_dummy_price_event(
    outbox: &Outbox<DummyEvent>,
    price: PriceOfOneBTC,
) -> anyhow::Result<()> {
    outbox
        .publish_ephemeral(
            PRICE_UPDATED_EVENT_TYPE,
            CorePriceEvent::PriceUpdated {
                price,
                timestamp: Utc::now(),
            },
        )
        .await?;

    Ok(())
}
