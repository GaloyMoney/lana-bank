use async_graphql::*;

use crate::{
    graphql::{
        credit_facility::balance::CollateralBalance, custody::Wallet, loader::LanaDataLoader,
        terms::TermsInput,
    },
    primitives::*,
};

pub use lana_app::credit::CreditFacilityProposal as DomainCreditFacilityProposal;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct CreditFacilityProposal {
    id: ID,
    credit_facility_proposal_id: UUID,
    approval_process_id: UUID,
    created_at: Timestamp,
    collateralization_state: CreditFacilityProposalCollateralizationState,
    facility_amount: UsdCents,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainCreditFacilityProposal>,
}

#[ComplexObject]
impl CreditFacilityProposal {
    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility propsal has collateral");

        if let Some(wallet_id) = collateral.wallet_id {
            Ok(loader.load_one(WalletId::from(wallet_id)).await?)
        } else {
            Ok(None)
        }
    }

    async fn collateral(&self, ctx: &Context<'_>) -> async_graphql::Result<CollateralBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let collateral = app
            .credit()
            .credit_facility_proposals()
            .collateral(sub, self.entity.id)
            .await?;

        Ok(CollateralBalance {
            btc_balance: collateral,
        })
    }
}

impl From<DomainCreditFacilityProposal> for CreditFacilityProposal {
    fn from(credit_facility_proposal: DomainCreditFacilityProposal) -> Self {
        let created_at = credit_facility_proposal.created_at();

        Self {
            id: credit_facility_proposal.id.to_global_id(),
            credit_facility_proposal_id: UUID::from(credit_facility_proposal.id),
            approval_process_id: UUID::from(credit_facility_proposal.approval_process_id),
            created_at: created_at.into(),
            facility_amount: credit_facility_proposal.amount,
            collateralization_state: credit_facility_proposal.last_collateralization_state(),

            entity: Arc::new(credit_facility_proposal),
        }
    }
}

#[derive(InputObject)]
pub struct CreditFacilityProposalCreateInput {
    pub customer_id: UUID,
    pub disbursal_credit_account_id: UUID,
    pub facility: UsdCents,
    pub terms: TermsInput,
    pub custodian_id: Option<UUID>,
}
crate::mutation_payload! { CreditFacilityProposalCreatePayload, credit_facility_proposal: CreditFacilityProposal }

#[derive(InputObject)]
pub struct CreditFacilityProposalCollateralUpdateInput {
    pub credit_facility_proposal_id: UUID,
    pub collateral: Satoshis,
    pub effective: Date,
}
crate::mutation_payload! { CreditFacilityProposalCollateralUpdatePayload, credit_facility_proposal: CreditFacilityProposal }
