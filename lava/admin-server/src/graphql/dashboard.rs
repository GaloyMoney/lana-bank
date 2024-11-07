use async_graphql::*;

use lava_app::dashboard::DashboardValues;

use crate::primitives::{Satoshis, UsdCents};

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
            total_disbursed: values.total_disbursed.into(),
            total_collateral: values.total_collateral.into(),
        }
    }
}
