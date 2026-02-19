use async_graphql::*;

use crate::primitives::*;

use super::loader::LanaDataLoader;

pub use admin_graphql_customer::{
    DomainProspect, ProspectBase, ProspectCloseInput, ProspectConvertInput, ProspectCreateInput,
    ProspectStatus,
};

// ===== Prospect =====

#[derive(Clone)]
pub(super) struct ProspectCrossDomain {
    entity: Arc<DomainProspect>,
}

#[Object]
impl ProspectCrossDomain {
    async fn email(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.email.clone())
    }

    async fn telegram_handle(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.telegram_handle.clone())
    }

    async fn customer_type(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<lana_app::customer::CustomerType> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.customer_type)
    }

    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<lana_app::customer::PersonalInfo>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.personal_info.clone())
    }

    async fn customer(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<super::customer::Customer>> {
        if self.entity.status != ProspectStatus::Converted {
            return Ok(None);
        }
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer_id = CustomerId::from(self.entity.id);
        let customer = loader.load_one(customer_id).await?;
        Ok(customer)
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "Prospect")]
pub struct Prospect(pub ProspectBase, ProspectCrossDomain);

impl From<DomainProspect> for Prospect {
    fn from(prospect: DomainProspect) -> Self {
        let base = ProspectBase::from(prospect);
        let cross = ProspectCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Prospect {
    type Target = ProspectBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { ProspectCreatePayload, prospect: Prospect }
crate::mutation_payload! { ProspectClosePayload, prospect: Prospect }
crate::mutation_payload! { ProspectConvertPayload, customer: super::customer::Customer }
