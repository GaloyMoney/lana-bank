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

#[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
#[serde(tag = "module")]
pub enum DummyEvent {
    Price(CorePriceEvent),
    #[serde(other)]
    Unknown,
}

pub async fn wait_for_price_to_be_updated(
    price: &Price,
    expected_price: PriceOfOneBTC,
) -> anyhow::Result<PriceOfOneBTC> {
    let deadline = Duration::from_secs(10);
    let poll_interval = Duration::from_millis(50);

    let result = tokio::time::timeout(deadline, async {
        loop {
            let current = price.usd_cents_per_btc().await;
            if current == expected_price {
                return current;
            }
            sleep(poll_interval).await;
        }
    })
    .await;

    match result {
        Ok(price) => Ok(price),
        Err(_) => Ok(price.usd_cents_per_btc().await),
    }
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
