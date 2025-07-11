use std::sync::Arc;

use async_graphql::{ComplexObject, Context, ID, Result, SimpleObject, Union};

use crate::{graphql::customer::Customer, primitives::*};
use lana_app::public_id::{PublicId as PublicIdScalar, PublicIdEntity};

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
}
