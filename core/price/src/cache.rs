use parking_lot::RwLock;
use std::sync::Arc;

use crate::PriceOfOneBTC;

#[derive(Clone)]
pub struct PriceCache {
    inner: Arc<RwLock<PriceOfOneBTC>>,
}

impl PriceCache {
    pub fn new(initial: PriceOfOneBTC) -> Self {
        Self {
            inner: Arc::new(RwLock::new(initial)),
        }
    }

    pub fn set_price(&self, price: PriceOfOneBTC) {
        *self.inner.write() = price;
    }

    pub fn get_price(&self) -> PriceOfOneBTC {
        *self.inner.read()
    }
}
