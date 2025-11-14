use crate::sources::PriceClient;
use crate::{PriceError, PriceOfOneBTC};

pub struct PriceAggregator<'a> {
    clients: Vec<&'a dyn PriceClient>,
}

impl<'a> PriceAggregator<'a> {
    pub fn new(clients: Vec<&'a dyn PriceClient>) -> Self {
        Self { clients }
    }

    pub async fn aggregate_btc_usd_price(&self) -> Result<PriceOfOneBTC, PriceError> {
        let mut quotes: Vec<PriceOfOneBTC> = Vec::new();
        for client in &self.clients {
            match client.fetch_btc_usd_price().await {
                Ok(price) => quotes.push(price),
                Err(e) => {
                    tracing::warn!(error = ?e, "price client failed");
                }
            }
        }

        if quotes.is_empty() {
            return Err(PriceError::NoPriceSourcesAvailable);
        }

        quotes.sort_by_key(|price| price.into_inner());
        let mid = quotes.len() / 2;
        let median_price = quotes.into_iter().nth(mid).unwrap();
        Ok(median_price)
    }
}
