use async_graphql::*;

use crate::primitives::*;

pub use lana_app::{
    customer::{CustomerType, Prospect as DomainProspect, ProspectStage, ProspectStatus},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
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
