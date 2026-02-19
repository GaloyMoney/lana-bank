#![allow(clippy::upper_case_acronyms)]

use async_graphql::*;
use serde::{Deserialize, Serialize};

pub use lana_app::primitives::{CustodianId, Subject, UserId, WalletId};

pub use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AdminAuthContext {
    pub sub: Subject,
}

pub use es_entity::graphql::UUID;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
scalar!(Timestamp);
impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}

pub trait ToGlobalId {
    fn to_global_id(&self) -> async_graphql::types::ID;
}

macro_rules! impl_to_global_id {
    ($($ty:ty),*) => {
        $(
            impl ToGlobalId for $ty {
                fn to_global_id(&self) -> async_graphql::types::ID {
                    async_graphql::types::ID::from(format!("{}:{}", stringify!($ty).trim_end_matches("Id"), self))
                }
            }
        )*
    };
}

impl_to_global_id! {
    CustodianId,
    WalletId
}
