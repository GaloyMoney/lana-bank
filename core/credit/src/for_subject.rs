use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;

use super::*;
use crate::history::CreditFacilityHistoryEntry;

pub struct CreditFacilitiesForSubject<'a, Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    customer_id: CustomerId,
    subject: &'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    authz: &'a Perms,
    credit_facilities: &'a CreditFacilities<Perms, E>,
    obligations: &'a Obligations<Perms, E>,
    disbursals: &'a Disbursals<Perms, E>,
    histories: &'a HistoryRepo,
    repayment_plans: &'a RepaymentPlanRepo,
    ledger: &'a CreditLedger,
}

impl<'a, Perms, E> CreditFacilitiesForSubject<'a, Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub(super) fn new(
        subject: &'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: CustomerId,
        authz: &'a Perms,
        credit_facilities: &'a CreditFacilities<Perms, E>,
        obligations: &'a Obligations<Perms, E>,
        disbursals: &'a Disbursals<Perms, E>,
        history: &'a HistoryRepo,
        repayment_plans: &'a RepaymentPlanRepo,
        ledger: &'a CreditLedger,
    ) -> Self {
        Self {
            customer_id,
            subject,
            authz,
            credit_facilities,
            obligations,
            disbursals,
            histories: history,
            repayment_plans,
            ledger,
        }
    }

    pub async fn list_credit_facilities_by_created_at(
        &self,
        query: PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: ListDirection,
    ) -> Result<PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>, CoreCreditError>
    {
        Ok(self
            .credit_facilities
            .list_for_customer(self.subject, self.customer_id, query, direction)
            .await?)
    }

    pub async fn history<T: From<CreditFacilityHistoryEntry>>(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CoreCreditError> {
        let id = id.into();
        let credit_facility = self.credit_facilities.find_by_id_without_audit(id).await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::credit_facility(id),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;
        let history = self.histories.load(id).await?;
        Ok(history.into_iter().map(T::from).collect())
    }

    pub async fn repayment_plan<T: From<CreditFacilityRepaymentPlanEntry>>(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CoreCreditError> {
        let id = id.into();
        let credit_facility = self.credit_facilities.find_by_id_without_audit(id).await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::credit_facility(id),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;
        let repayment_plan = self.repayment_plans.load(id).await?;
        Ok(repayment_plan.entries.into_iter().map(T::from).collect())
    }

    pub async fn balance(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacilityBalanceSummary, CoreCreditError> {
        let id = id.into();
        let credit_facility = self.credit_facilities.find_by_id_without_audit(id).await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::credit_facility(id),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;

        let balances = self
            .ledger
            .get_credit_facility_balance(credit_facility.account_ids)
            .await?;

        Ok(balances)
    }

    pub async fn find_by_id(
        &self,
        id: impl Into<CreditFacilityId>,
    ) -> Result<Option<CreditFacility>, CoreCreditError> {
        let id = id.into();
        match self.credit_facilities.find_by_id_without_audit(id).await {
            Ok(cf) => {
                self.ensure_credit_facility_access(
                    &cf,
                    CoreCreditObject::credit_facility(id),
                    CoreCreditAction::CREDIT_FACILITY_READ,
                )
                .await?;
                Ok(Some(cf))
            }
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn ensure_credit_facility_access(
        &self,
        credit_facility: &CreditFacility,
        object: CoreCreditObject,
        action: CoreCreditAction,
    ) -> Result<(), CoreCreditError> {
        if credit_facility.customer_id != self.customer_id {
            self.authz
                .audit()
                .record_entry(self.subject, object, action, false)
                .await?;
            return Err(CoreCreditError::CustomerMismatchForCreditFacility);
        }

        self.authz
            .audit()
            .record_entry(self.subject, object, action, true)
            .await?;
        Ok(())
    }

    pub async fn list_disbursals_for_credit_facility(
        &self,
        id: CreditFacilityId,
        query: es_entity::PaginatedQueryArgs<DisbursalsCursor>,
        sort: impl Into<Sort<DisbursalsSortBy>>,
    ) -> Result<es_entity::PaginatedQueryRet<Disbursal, DisbursalsCursor>, CoreCreditError> {
        let credit_facility = self.credit_facilities.find_by_id_without_audit(id).await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::DISBURSAL_LIST,
        )
        .await?;
        let disbursals = self
            .disbursals
            .list_for_facility_without_audit(id, query, sort)
            .await?;

        Ok(disbursals)
    }

    pub async fn find_disbursal_by_concluded_tx_id(
        &self,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, CoreCreditError> {
        let tx_id = tx_id.into();
        let disbursal = self
            .disbursals
            .find_by_concluded_tx_id_without_audit(tx_id)
            .await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id_without_audit(disbursal.facility_id)
            .await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;

        Ok(disbursal)
    }

    pub async fn find_payment_allocation_by_id(
        &self,
        payment_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, CoreCreditError> {
        let allocation = self
            .obligations
            .find_allocation_by_id_without_audit(payment_id.into())
            .await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id_without_audit(allocation.credit_facility_id)
            .await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;

        Ok(allocation)
    }
}
