use async_graphql::*;

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
    customer_type: CustomerType,

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
            customer_type: prospect.customer_type,
            entity: Arc::new(prospect),
        }
    }
}

#[ComplexObject]
impl Prospect {
    async fn stage(&self) -> ProspectStage {
        self.entity.stage
    }

    async fn email(&self) -> &str {
        &self.entity.email
    }

    async fn telegram_handle(&self) -> &str {
        &self.entity.telegram_handle
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

    async fn personal_info(&self) -> Option<&PersonalInfo> {
        self.entity.personal_info.as_ref()
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
