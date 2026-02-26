use async_graphql::*;

use admin_graphql_shared::primitives::{Satoshis, UsdCents};
use lana_app::dashboard::DashboardValues;

#[derive(SimpleObject)]
pub struct Dashboard {
    active_facilities: u32,
    pending_facilities: u32,
    total_disbursed: UsdCents,
    total_collateral: Satoshis,
}

impl From<DashboardValues> for Dashboard {
    fn from(values: DashboardValues) -> Self {
        Dashboard {
            active_facilities: values.active_facilities,
            pending_facilities: values.pending_facilities,
            total_disbursed: values.total_disbursed,
            total_collateral: values.total_collateral,
        }
    }
}
