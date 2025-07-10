use std::sync::Arc;

use async_graphql::{ComplexObject, Context, ID, Result, SimpleObject, Union};

use crate::{graphql::customer::Customer, primitives::*};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PublicRef {
    pub id: ID,
    pub target_id: UUID,
    pub reference: String,
    pub target_type: String,
    #[graphql(skip)]
    #[allow(dead_code)]
    entity: Arc<lana_app::public_ref::PublicRef>,
}

impl From<lana_app::public_ref::PublicRef> for PublicRef {
    fn from(public_ref: lana_app::public_ref::PublicRef) -> Self {
        PublicRef {
            id: public_ref.id.to_global_id(),
            target_id: UUID::from(public_ref.target_id),
            reference: public_ref.id.to_string(),
            target_type: public_ref.target_type.to_string(),
            entity: Arc::new(public_ref),
        }
    }
}

#[derive(Union)]
pub enum PublicRefTarget {
    Customer(Customer),
}

#[ComplexObject]
impl PublicRef {
    pub async fn target(&self, ctx: &Context<'_>) -> Result<Option<PublicRefTarget>> {
        // Check the target type and fetch the appropriate entity
        match self.target_type.as_str() {
            "customer" => {
                // Since the target_id is the customer id use that to call the GQL
                // data loader to lookup the customer
                let customer_id = CustomerId::from(self.target_id);
                let loader = ctx.data_unchecked::<async_graphql::dataloader::DataLoader<crate::graphql::loader::LanaLoader>>();

                if let Ok(Some(customer)) = loader.load_one(customer_id).await {
                    Ok(Some(PublicRefTarget::Customer(customer)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
