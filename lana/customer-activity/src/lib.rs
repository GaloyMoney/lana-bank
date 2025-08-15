#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
pub mod job;
mod primitives;
mod repo;

use error::*;

pub use job::{CustomerActivityProjectionConfig, CustomerActivityProjectionInit};
pub use primitives::CustomerActivity;
pub use repo::CustomerActivityRepo;

#[derive(Clone)]
pub struct CustomerActivityService {
    repo: CustomerActivityRepo,
}

impl CustomerActivityService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            repo: CustomerActivityRepo::new(pool),
        }
    }

    pub async fn load_activity_by_customer_id(
        &self,
        customer_id: core_customer::CustomerId,
    ) -> Result<Option<CustomerActivity>, CustomerActivityError> {
        Ok(self.repo.load_activity_by_customer_id(customer_id).await?)
    }

    pub async fn persist_activity(
        &self,
        activity: &CustomerActivity,
    ) -> Result<(), CustomerActivityError> {
        Ok(self.repo.persist_activity(activity).await?)
    }

    pub async fn find_customers_with_activity_in_range(
        &self,
        start_threshold: chrono::DateTime<chrono::Utc>,
        end_threshold: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<core_customer::CustomerId>, CustomerActivityError> {
        Ok(self
            .repo
            .find_customers_with_activity_in_range(start_threshold, end_threshold)
            .await?)
    }

    pub async fn find_customers_without_activity(
        &self,
    ) -> Result<Vec<core_customer::CustomerId>, CustomerActivityError> {
        Ok(self.repo.find_customers_without_activity().await?)
    }
}
