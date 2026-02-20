use audit::SystemSubject;

use crate::{
    chart_of_accounts_integration::ResolvedChartOfAccountsIntegrationConfig,
    collateral::ledger::CollateralAccountSets,
    pending_credit_facility::PendingCreditFacility,
    primitives::{
        CalaAccountId, DisbursalId, LedgerOmnibusAccountIds, LedgerTxId, Satoshis, UsdCents,
    },
};

use core_credit_collection::Obligation;

use super::{
    LongTermDisbursedIntegrationMeta, ShortTermDisbursedIntegrationMeta,
    balance::{CreditFacilityBalanceSummary, PendingCreditFacilityBalanceSummary},
    credit_facility_accounts::{
        CreditFacilityActivation, CreditFacilityInterestAccrual,
        CreditFacilityInterestAccrualCycle, CreditFacilityLedgerAccountIds,
        PendingCreditFacilityAccountIds,
    },
    error::CreditLedgerError,
};

pub trait CreditLedgerOps: Clone + Send + Sync + 'static {
    // Balance queries
    fn get_pending_credit_facility_balance(
        &self,
        account_ids: PendingCreditFacilityAccountIds,
        collateral_account_id: CalaAccountId,
    ) -> impl std::future::Future<
        Output = Result<PendingCreditFacilityBalanceSummary, CreditLedgerError>,
    > + Send;

    fn get_collateral_for_pending_facility(
        &self,
        collateral_account_id: CalaAccountId,
    ) -> impl std::future::Future<Output = Result<Satoshis, CreditLedgerError>> + Send;

    fn get_credit_facility_balance(
        &self,
        account_ids: CreditFacilityLedgerAccountIds,
        collateral_account_id: CalaAccountId,
    ) -> impl std::future::Future<Output = Result<CreditFacilityBalanceSummary, CreditLedgerError>> + Send;

    // Facility lifecycle
    fn handle_pending_facility_creation_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        pending_credit_facility: &PendingCreditFacility,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn handle_activation_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        activation: CreditFacilityActivation,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    // Interest
    fn record_interest_accrual_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        accrual: CreditFacilityInterestAccrual,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn record_interest_accrual_cycle_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        cycle: CreditFacilityInterestAccrualCycle,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    // Disbursals
    fn initiate_disbursal_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        entity_id: DisbursalId,
        tx_id: LedgerTxId,
        amount: UsdCents,
        account_ids: CreditFacilityLedgerAccountIds,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn cancel_disbursal_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        entity_id: DisbursalId,
        tx_id: LedgerTxId,
        amount: UsdCents,
        account_ids: CreditFacilityLedgerAccountIds,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn settle_disbursal_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        entity_id: DisbursalId,
        disbursed_into_account_id: CalaAccountId,
        obligation: Obligation,
        account_ids: CreditFacilityLedgerAccountIds,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    // Chart of accounts integration
    fn attach_chart_of_accounts_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        new_integration_config: &ResolvedChartOfAccountsIntegrationConfig,
        old_integration_config: Option<&ResolvedChartOfAccountsIntegrationConfig>,
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn attach_short_term_disbursed_receivable_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        new_integration_meta: &ShortTermDisbursedIntegrationMeta,
        old_integration_meta: Option<&ShortTermDisbursedIntegrationMeta>,
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    fn attach_long_term_disbursed_receivable_account_sets_in_op(
        &self,
        op: &mut es_entity::DbOpWithTime<'_>,
        new_integration_meta: &LongTermDisbursedIntegrationMeta,
        old_integration_meta: Option<&LongTermDisbursedIntegrationMeta>,
    ) -> impl std::future::Future<Output = Result<(), CreditLedgerError>> + Send;

    // Accessors
    fn collateral_account_sets(&self) -> CollateralAccountSets;
    fn liquidation_proceeds_omnibus_account_ids(&self) -> &LedgerOmnibusAccountIds;
    fn collateral_omnibus_account_ids(&self) -> &LedgerOmnibusAccountIds;
    fn payments_made_omnibus_account_ids(&self) -> &LedgerOmnibusAccountIds;
}
