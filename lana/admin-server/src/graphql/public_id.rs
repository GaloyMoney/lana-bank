use std::sync::Arc;

use async_graphql::{ComplexObject, Context, ID, Result, SimpleObject, Union};

use crate::{graphql::customer::Customer, primitives::*};
use lana_app::public_id::{PublicId as PublicIdScalar, PublicIdEntity};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct PublicId {
    pub id: ID,
    pub target_id: UUID,
    pub reference: PublicIdScalar,
    pub target_type: String,
    #[graphql(skip)]
    #[allow(dead_code)]
    entity: Arc<PublicIdEntity>,
}

impl From<PublicIdEntity> for PublicId {
    fn from(public_id_entity: PublicIdEntity) -> Self {
        PublicId {
            id: public_id_entity.id.to_global_id(),
            target_id: UUID::from(public_id_entity.target_id),
            reference: public_id_entity.id.clone(),
            target_type: public_id_entity.target_type.to_string(),
            entity: Arc::new(public_id_entity),
        }
    }
}

#[derive(Union)]
pub enum PublicIdTarget {
    Customer(Customer),
}

#[ComplexObject]
impl PublicId {
    pub async fn target(&self, ctx: &Context<'_>) -> Result<Option<PublicIdTarget>> {
        // Check the target type and fetch the appropriate entity
        match self.target_type.as_str() {
            "customer" => {
                // Since the target_id is the customer id use that to call the GQL
                // data loader to lookup the customer
                let customer_id = CustomerId::from(self.target_id);
                let loader = ctx.data_unchecked::<async_graphql::dataloader::DataLoader<crate::graphql::loader::LanaLoader>>();

                if let Ok(Some(customer)) = loader.load_one(customer_id).await {
                    Ok(Some(PublicIdTarget::Customer(customer)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
