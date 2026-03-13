use async_graphql::{
    connection::{Connection, EmptyFields},
    *,
};

use crate::graphql::event_timeline::{self, EventTimelineCursor, EventTimelineEntry};
use crate::primitives::*;
use lana_app::public_id::PublicId;

use super::{loader::LanaDataLoader, primitives::SortDirection};

use es_entity::Sort;
use lana_app::customer::{
    CustomerType, KycLevel, KycStatus, PersonalInfo, ProspectStage,
    ProspectsSortBy as DomainProspectsSortBy,
};
pub use lana_app::customer::{
    Prospect as DomainProspect, ProspectsFilters as DomainProspectsFilters,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Prospect {
    id: ID,
    prospect_id: UUID,
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

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }

    async fn customer(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<super::customer::Customer>> {
        if self.entity.stage != ProspectStage::Converted {
            return Ok(None);
        }
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer_id = CustomerId::from(self.entity.id);
        let customer = loader.load_one(customer_id).await?;
        Ok(customer)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ProspectKycUpdatedPayload {
    pub kyc_status: KycStatus,
    #[graphql(skip)]
    pub prospect_id: ProspectId,
}

#[ComplexObject]
impl ProspectKycUpdatedPayload {
    async fn prospect(&self, ctx: &Context<'_>) -> async_graphql::Result<Prospect> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let prospect = loader
            .load_one(self.prospect_id)
            .await?
            .expect("Prospect not found");
        Ok(prospect)
    }
}

#[derive(InputObject)]
pub struct ProspectCreateInput {
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
}
crate::mutation_payload! { ProspectCreatePayload, prospect: Prospect }

#[derive(InputObject)]
pub struct ProspectCloseInput {
    pub prospect_id: UUID,
}
crate::mutation_payload! { ProspectClosePayload, prospect: Prospect }

#[derive(InputObject)]
pub struct ProspectConvertInput {
    pub prospect_id: UUID,
}
crate::mutation_payload! { ProspectConvertPayload, customer: super::customer::Customer }

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProspectsSortBy {
    #[default]
    CreatedAt,
    Email,
    TelegramHandle,
}

impl From<ProspectsSortBy> for DomainProspectsSortBy {
    fn from(by: ProspectsSortBy) -> Self {
        match by {
            ProspectsSortBy::CreatedAt => DomainProspectsSortBy::CreatedAt,
            ProspectsSortBy::Email => DomainProspectsSortBy::Email,
            ProspectsSortBy::TelegramHandle => DomainProspectsSortBy::TelegramHandle,
        }
    }
}

#[derive(InputObject, Default, Clone, Copy)]
pub struct ProspectsSort {
    #[graphql(default)]
    pub by: ProspectsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<ProspectsSort> for Sort<DomainProspectsSortBy> {
    fn from(sort: ProspectsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<ProspectsSort> for DomainProspectsSortBy {
    fn from(sort: ProspectsSort) -> Self {
        sort.by.into()
    }
}

#[derive(InputObject)]
pub struct ProspectsFilter {
    pub stage: Option<ProspectStage>,
    pub customer_type: Option<CustomerType>,
}
