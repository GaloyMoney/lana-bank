mod ledger_accounts;

use async_graphql::*;

use crate::{
    graphql::{
        accounting::LedgerTransaction, approval_process::ApprovalProcess, custody::*, customer::*,
        loader::LanaDataLoader,
    },
    primitives::*,
};

// Re-export base types and value types from the credit crate
pub use admin_graphql_credit::{
    CollateralBase, CollateralRecordProceedsFromLiquidationInput,
    CollateralRecordSentToLiquidationInput, CollateralUpdateInput, CreditFacilitiesCursor,
    CreditFacilitiesFilter, CreditFacilitiesSort, CreditFacilityBase,
    CreditFacilityCollateralizationUpdated, CreditFacilityCompleteInput,
    CreditFacilityDisbursalBase, CreditFacilityDisbursalInitiateInput,
    CreditFacilityPartialPaymentRecordInput, CreditFacilityPartialPaymentWithDateRecordInput,
    CreditFacilityPaymentAllocationBase, CreditFacilityProposalBase,
    CreditFacilityProposalCreateInput, CreditFacilityProposalCustomerApprovalConcludeInput,
    CreditFacilityProposalsByCreatedAtCursor, DisbursalsCursor, DisbursalsFilters,
    DomainCollateral, DomainCreditFacilitiesFilters, DomainCreditFacilitiesSortBy,
    DomainCreditFacility, DomainCreditFacilityProposal, DomainDisbursal, DomainDisbursalsSortBy,
    DomainLiquidation, DomainPendingCreditFacility, LiquidationBase, ListDirection,
    PendingCreditFacilitiesByCreatedAtCursor, PendingCreditFacilityBase,
    PendingCreditFacilityCollateralizationUpdated, Sort,
};

use lana_app::custody::WalletId;
use ledger_accounts::*;

// ===== CreditFacility =====

#[derive(Clone)]
pub(super) struct CreditFacilityCrossDomain {
    entity: Arc<DomainCreditFacility>,
}

#[Object]
impl CreditFacilityCrossDomain {
    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility has collateral");

        if let Some(wallet_id) = collateral.wallet_id {
            Ok(loader.load_one(WalletId::from(wallet_id)).await?)
        } else {
            Ok(None)
        }
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<CreditFacilityDisbursal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let disbursals = app
            .credit()
            .disbursals()
            .list(
                sub,
                Default::default(),
                DisbursalsFilters {
                    credit_facility_id: Some(self.entity.id),
                    ..Default::default()
                },
                Sort {
                    by: DomainDisbursalsSortBy::CreatedAt,
                    direction: ListDirection::Descending,
                },
            )
            .await?;

        Ok(disbursals
            .entities
            .into_iter()
            .map(CreditFacilityDisbursal::from)
            .collect())
    }

    async fn liquidations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Liquidation>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let liquidations = app
            .credit()
            .collaterals()
            .list_liquidations_for_collateral_by_created_at(
                sub,
                self.entity.collateral_id,
                Default::default(),
            )
            .await?;

        Ok(liquidations
            .entities
            .into_iter()
            .map(Liquidation::from)
            .collect())
    }

    async fn ledger_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityLedgerAccounts> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility has collateral");

        Ok(CreditFacilityLedgerAccounts {
            facility_account_id: self.entity.account_ids.facility_account_id.into(),
            disbursed_receivable_not_yet_due_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_not_yet_due_account_id
                .into(),
            disbursed_receivable_due_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_due_account_id
                .into(),
            disbursed_receivable_overdue_account_id: self
                .entity
                .account_ids
                .disbursed_receivable_overdue_account_id
                .into(),
            disbursed_defaulted_account_id: self
                .entity
                .account_ids
                .disbursed_defaulted_account_id
                .into(),
            collateral_account_id: collateral.entity.account_ids.collateral_account_id.into(),
            collateral_in_liquidation_account_id: collateral
                .entity
                .account_ids
                .collateral_in_liquidation_account_id
                .into(),
            liquidated_collateral_account_id: collateral
                .entity
                .account_ids
                .liquidated_collateral_account_id
                .into(),
            proceeds_from_liquidation_account_id: self
                .entity
                .account_ids
                .proceeds_from_liquidation_account_id
                .into_inner()
                .into(),
            interest_receivable_not_yet_due_account_id: self
                .entity
                .account_ids
                .interest_receivable_not_yet_due_account_id
                .into(),
            interest_receivable_due_account_id: self
                .entity
                .account_ids
                .interest_receivable_due_account_id
                .into(),
            interest_receivable_overdue_account_id: self
                .entity
                .account_ids
                .interest_receivable_overdue_account_id
                .into(),
            interest_defaulted_account_id: self
                .entity
                .account_ids
                .interest_defaulted_account_id
                .into(),
            interest_income_account_id: self.entity.account_ids.interest_income_account_id.into(),
            fee_income_account_id: self.entity.account_ids.fee_income_account_id.into(),
            payment_holding_account_id: self.entity.account_ids.payment_holding_account_id.into(),
            uncovered_outstanding_account_id: self
                .entity
                .account_ids
                .uncovered_outstanding_account_id
                .into(),
        })
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "CreditFacility")]
pub struct CreditFacility(pub CreditFacilityBase, CreditFacilityCrossDomain);

impl From<DomainCreditFacility> for CreditFacility {
    fn from(cf: DomainCreditFacility) -> Self {
        let base = CreditFacilityBase::from(cf);
        let cross = CreditFacilityCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for CreditFacility {
    type Target = CreditFacilityBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { CreditFacilityPartialPaymentRecordPayload, credit_facility: CreditFacility }
crate::mutation_payload! { CreditFacilityCompletePayload, credit_facility: CreditFacility }

// ===== CreditFacilityProposal =====

#[derive(Clone)]
pub(super) struct CreditFacilityProposalCrossDomain {
    entity: Arc<DomainCreditFacilityProposal>,
}

#[Object]
impl CreditFacilityProposalCrossDomain {
    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Custodian>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        if let Some(custodian_id) = self.entity.custodian_id {
            let custodian = loader
                .load_one(custodian_id)
                .await?
                .expect("custodian not found");

            return Ok(Some(custodian));
        }
        Ok(None)
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<ApprovalProcess>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        if let Some(approval_process_id) = self.entity.approval_process_id {
            let process = loader
                .load_one(approval_process_id)
                .await?
                .expect("process not found");
            return Ok(Some(process));
        }
        Ok(None)
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "CreditFacilityProposal")]
pub struct CreditFacilityProposal(
    pub CreditFacilityProposalBase,
    CreditFacilityProposalCrossDomain,
);

impl From<DomainCreditFacilityProposal> for CreditFacilityProposal {
    fn from(proposal: DomainCreditFacilityProposal) -> Self {
        let base = CreditFacilityProposalBase::from(proposal);
        let cross = CreditFacilityProposalCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for CreditFacilityProposal {
    type Target = CreditFacilityProposalBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { CreditFacilityProposalCreatePayload, credit_facility_proposal: CreditFacilityProposal }
crate::mutation_payload! { CreditFacilityProposalCustomerApprovalConcludePayload, credit_facility_proposal: CreditFacilityProposal }

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityProposalConcludedPayload {
    pub status: CreditFacilityProposalStatus,
    #[graphql(skip)]
    pub credit_facility_proposal_id: CreditFacilityProposalId,
}

#[ComplexObject]
impl CreditFacilityProposalConcludedPayload {
    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<CreditFacilityProposal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let proposal = loader
            .load_one(self.credit_facility_proposal_id)
            .await?
            .expect("credit facility proposal not found");
        Ok(proposal)
    }
}

// ===== PendingCreditFacility =====

#[derive(Clone)]
pub(super) struct PendingCreditFacilityCrossDomain {
    entity: Arc<DomainPendingCreditFacility>,
}

#[Object]
impl PendingCreditFacilityCrossDomain {
    async fn wallet(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Wallet>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("credit facility proposal has collateral");

        if let Some(wallet_id) = collateral.wallet_id {
            Ok(loader.load_one(WalletId::from(wallet_id)).await?)
        } else {
            Ok(None)
        }
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("customer not found");
        Ok(customer)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "PendingCreditFacility")]
pub struct PendingCreditFacility(
    pub PendingCreditFacilityBase,
    PendingCreditFacilityCrossDomain,
);

impl From<DomainPendingCreditFacility> for PendingCreditFacility {
    fn from(pending: DomainPendingCreditFacility) -> Self {
        let base = PendingCreditFacilityBase::from(pending);
        let cross = PendingCreditFacilityCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for PendingCreditFacility {
    type Target = PendingCreditFacilityBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCollateralizationPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}

#[derive(SimpleObject)]
pub struct PendingCreditFacilityCompleted {
    pub status: PendingCreditFacilityStatus,
    pub recorded_at: Timestamp,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PendingCreditFacilityCompletedPayload {
    #[graphql(flatten)]
    pub update: PendingCreditFacilityCompleted,
    #[graphql(skip)]
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

#[ComplexObject]
impl PendingCreditFacilityCompletedPayload {
    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<PendingCreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.pending_credit_facility_id)
            .await?
            .expect("pending credit facility not found");
        Ok(facility)
    }
}

// ===== CreditFacilityDisbursal =====

#[derive(Clone)]
pub(super) struct CreditFacilityDisbursalCrossDomain {
    entity: Arc<DomainDisbursal>,
}

#[Object]
impl CreditFacilityDisbursalCrossDomain {
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.entity.facility_id)
            .await?
            .expect("committee not found");
        Ok(facility)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn ledger_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<LedgerTransaction>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let tx_ids = self.entity.ledger_tx_ids();
        let loaded_transactions = loader.load_many(tx_ids.iter().copied()).await?;

        Ok(tx_ids
            .iter()
            .filter_map(|id| loaded_transactions.get(id).cloned())
            .collect())
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "CreditFacilityDisbursal")]
pub struct CreditFacilityDisbursal(
    pub CreditFacilityDisbursalBase,
    CreditFacilityDisbursalCrossDomain,
);

impl From<DomainDisbursal> for CreditFacilityDisbursal {
    fn from(disbursal: DomainDisbursal) -> Self {
        let base = CreditFacilityDisbursalBase::from(disbursal);
        let cross = CreditFacilityDisbursalCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for CreditFacilityDisbursal {
    type Target = CreditFacilityDisbursalBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { CreditFacilityDisbursalInitiatePayload, disbursal: CreditFacilityDisbursal }

// ===== Collateral =====

#[derive(Clone)]
pub(super) struct CollateralCrossDomain {
    entity: Arc<DomainCollateral>,
}

#[Object]
impl CollateralCrossDomain {
    async fn account(&self, ctx: &Context<'_>) -> Result<super::accounting::LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(LedgerAccountId::from(
                self.entity.account_ids.collateral_account_id,
            ))
            .await?
            .expect("Collateral account not found");
        Ok(collateral)
    }

    async fn credit_facility(&self, ctx: &Context<'_>) -> Result<Option<CreditFacility>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader.load_one(self.entity.credit_facility_id).await?;
        Ok(facility)
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "Collateral")]
pub struct Collateral(pub CollateralBase, CollateralCrossDomain);

impl From<DomainCollateral> for Collateral {
    fn from(collateral: DomainCollateral) -> Self {
        let base = CollateralBase::from(collateral);
        let cross = CollateralCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Collateral {
    type Target = CollateralBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { CollateralUpdatePayload, collateral: Collateral }
crate::mutation_payload! { CollateralRecordSentToLiquidationPayload, collateral: Collateral }
crate::mutation_payload! { CollateralRecordProceedsFromLiquidationPayload, collateral: Collateral }

// ===== Liquidation =====

#[derive(Clone)]
pub(super) struct LiquidationCrossDomain {
    entity: Arc<DomainLiquidation>,
}

#[Object]
impl LiquidationCrossDomain {
    async fn collateral(&self, ctx: &Context<'_>) -> Result<Collateral> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let collateral = loader
            .load_one(self.entity.collateral_id)
            .await?
            .expect("Collateral not found");
        Ok(collateral)
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "Liquidation")]
pub struct Liquidation(pub LiquidationBase, LiquidationCrossDomain);

impl From<DomainLiquidation> for Liquidation {
    fn from(liquidation: DomainLiquidation) -> Self {
        let base = LiquidationBase::from(liquidation);
        let cross = LiquidationCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Liquidation {
    type Target = LiquidationBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ===== CreditFacilityPaymentAllocation =====

#[derive(Clone)]
pub(super) struct CreditFacilityPaymentAllocationCrossDomain {
    entity: Arc<admin_graphql_credit::DomainPaymentAllocation>,
}

#[Object]
impl CreditFacilityPaymentAllocationCrossDomain {
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        let cf = app
            .credit()
            .for_subject(sub)?
            .find_by_id(self.entity.beneficiary_id)
            .await?
            .expect("facility should exist for a payment");
        Ok(CreditFacility::from(cf))
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "CreditFacilityPaymentAllocation")]
pub struct CreditFacilityPaymentAllocation(
    pub CreditFacilityPaymentAllocationBase,
    CreditFacilityPaymentAllocationCrossDomain,
);

impl From<admin_graphql_credit::DomainPaymentAllocation> for CreditFacilityPaymentAllocation {
    fn from(allocation: admin_graphql_credit::DomainPaymentAllocation) -> Self {
        let base = CreditFacilityPaymentAllocationBase::from(allocation);
        let cross = CreditFacilityPaymentAllocationCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for CreditFacilityPaymentAllocation {
    type Target = CreditFacilityPaymentAllocationBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ===== CreditFacilityCollateralizationPayload (subscription) =====

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CreditFacilityCollateralizationPayload {
    #[graphql(flatten)]
    pub update: CreditFacilityCollateralizationUpdated,
    #[graphql(skip)]
    pub credit_facility_id: CreditFacilityId,
}

#[ComplexObject]
impl CreditFacilityCollateralizationPayload {
    async fn credit_facility(&self, ctx: &Context<'_>) -> async_graphql::Result<CreditFacility> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let facility = loader
            .load_one(self.credit_facility_id)
            .await?
            .expect("credit facility not found");
        Ok(facility)
    }
}
