use async_graphql::*;

use crate::primitives::*;

pub use lana_app::{
    customer::{CustomerType, Prospect as DomainProspect, ProspectStage, ProspectStatus},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "Prospect", complex)]
pub struct ProspectBase {
    id: ID,
    prospect_id: UUID,
    status: ProspectStatus,
    kyc_status: lana_app::customer::KycStatus,
    level: lana_app::customer::KycLevel,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainProspect>,
}

impl From<DomainProspect> for ProspectBase {
    fn from(prospect: DomainProspect) -> Self {
        ProspectBase {
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
impl ProspectBase {
    async fn stage(&self) -> ProspectStage {
        self.entity.stage
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

    async fn email(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let parties: std::collections::HashMap<_, std::sync::Arc<lana_app::customer::Party>> = app
            .customers()
            .find_all_parties(&[self.entity.party_id])
            .await?;
        let party = parties
            .into_values()
            .next()
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.email.clone())
    }

    async fn telegram_handle(&self, ctx: &Context<'_>) -> async_graphql::Result<String> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let parties: std::collections::HashMap<_, std::sync::Arc<lana_app::customer::Party>> = app
            .customers()
            .find_all_parties(&[self.entity.party_id])
            .await?;
        let party = parties
            .into_values()
            .next()
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.telegram_handle.clone())
    }

    async fn customer_type(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<lana_app::customer::CustomerType> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let parties: std::collections::HashMap<_, std::sync::Arc<lana_app::customer::Party>> = app
            .customers()
            .find_all_parties(&[self.entity.party_id])
            .await?;
        let party = parties
            .into_values()
            .next()
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.customer_type)
    }

    async fn personal_info(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<lana_app::customer::PersonalInfo>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let parties: std::collections::HashMap<_, std::sync::Arc<lana_app::customer::Party>> = app
            .customers()
            .find_all_parties(&[self.entity.party_id])
            .await?;
        let party = parties
            .into_values()
            .next()
            .ok_or_else(|| async_graphql::Error::new("Party not found"))?;
        Ok(party.personal_info.clone())
    }

    async fn customer(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<crate::customer::CustomerBase>> {
        if self.entity.status != ProspectStatus::Converted {
            return Ok(None);
        }
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let customers: std::collections::HashMap<_, crate::customer::CustomerBase> = app
            .customers()
            .find_all(&[lana_app::primitives::CustomerId::from(self.entity.id)])
            .await?;
        Ok(customers.into_values().next())
    }
}

#[derive(InputObject)]
pub struct ProspectCreateInput {
    pub email: String,
    pub telegram_handle: String,
    pub customer_type: CustomerType,
}

#[derive(InputObject)]
pub struct ProspectCloseInput {
    pub prospect_id: UUID,
}

#[derive(InputObject)]
pub struct ProspectConvertInput {
    pub prospect_id: UUID,
}
