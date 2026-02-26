use async_graphql::*;

use crate::Customer;
use crate::LanaDataLoader;
use crate::primitives::*;
use lana_app::public_id::PublicId;

pub use lana_app::customer::Prospect as DomainProspect;
use lana_app::customer::{
    CustomerType, KycLevel, KycStatus, PersonalInfo, ProspectStage, ProspectStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Prospect {
    id: ID,
    prospect_id: UUID,
    status: ProspectStatus,
    kyc_status: KycStatus,
    level: KycLevel,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainProspect>,
}

impl From<DomainProspect> for Prospect {
    fn from(prospect: DomainProspect) -> Self {
        Prospect {
            id: prospect.id.to_global_id(),
            prospect_id: UUID::from(prospect.id),
            status: prospect.status,
            kyc_status: prospect.kyc_status,
            level: prospect.level,
            created_at: prospect.created_at().into(),
            entity: Arc::new(prospect),
        }
    }
}

#[ComplexObject]
impl Prospect {
    async fn stage(&self) -> ProspectStage {
        self.entity.stage
    }

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

    async fn customer_type(&self, ctx: &Context<'_>) -> async_graphql::Result<CustomerType> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.customer_type)
    }

    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn applicant_id(&self) -> Option<&str> {
        self.entity.applicant_id.as_deref()
    }

    async fn verification_link(&self) -> Option<&str> {
        self.entity.verification_link.as_deref()
    }

    async fn verification_link_created_at(&self) -> Option<Timestamp> {
        self.entity
            .verification_link_created_at()
            .map(Timestamp::from)
    }

    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<PersonalInfo>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let party = loader
            .load_one(self.entity.party_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.personal_info.clone())
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Customer>> {
        if self.entity.status != ProspectStatus::Converted {
            return Ok(None);
        }
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer_id = CustomerId::from(self.entity.id);
        let customer = loader.load_one(customer_id).await?;
        Ok(customer)
    }
}

#[derive(InputObject)]
pub struct ProspectCreateInput {
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
}
mutation_payload! { ProspectCreatePayload, prospect: Prospect }

#[derive(InputObject)]
pub struct ProspectCloseInput {
    pub prospect_id: UUID,
}
mutation_payload! { ProspectClosePayload, prospect: Prospect }

#[derive(InputObject)]
pub struct ProspectConvertInput {
    pub prospect_id: UUID,
}
mutation_payload! { ProspectConvertPayload, customer: Customer }
