use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};

use super::*;
use crate::history::CreditFacilityHistoryEntry;

pub struct CreditFacilitiesForSubject<'a, Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    customer_id: CustomerId,
    subject: &'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    authz: &'a Perms,
    credit_facilities: &'a CreditFacilityRepo<E>,
    disbursals: &'a DisbursalRepo<E>,
    payments: &'a PaymentRepo,
    histories: &'a HistoryRepo,
    ledger: &'a CreditLedger,
}

impl<'a, Perms, E> CreditFacilitiesForSubject<'a, Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        subject: &'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: CustomerId,
        authz: &'a Perms,
        credit_facilities: &'a CreditFacilityRepo<E>,
        disbursals: &'a DisbursalRepo<E>,
        payments: &'a PaymentRepo,
        history: &'a HistoryRepo,
        ledger: &'a CreditLedger,
    ) -> Self {
        Self {
            customer_id,
            subject,
            authz,
            credit_facilities,
            disbursals,
            payments,
            histories: history,
            ledger,
        }
    }

    pub async fn list_credit_facilities_by_created_at(
        &self,
        query: PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: ListDirection,
    ) -> Result<PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>, CoreCreditError>
    {
        self.authz
            .audit()
            .record_entry(
                self.subject,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
                true,
            )
            .await?;

        Ok(self
            .credit_facilities
            .list_for_customer_id_by_created_at(self.customer_id, query, direction)
            .await?)
    }

    pub async fn history<T: From<CreditFacilityHistoryEntry>>(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<Vec<T>, CoreCreditError> {
        let id = id.into();
        let credit_facility = self.credit_facilities.find_by_id(id).await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::credit_facility(id),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;
        let history = self.histories.load(id).await?;
        Ok(history.entries.into_iter().rev().map(T::from).collect())
    }

    pub async fn balance(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacilityBalanceSummary, CoreCreditError> {
        let id = id.into();
        let credit_facility = self.credit_facilities.find_by_id(id).await?;

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
        match self.credit_facilities.find_by_id(id).await {
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
        let credit_facility = self.credit_facilities.find_by_id(id).await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::DISBURSAL_LIST,
        )
        .await?;

        let disbursals = self
            .disbursals
            .find_many(
                FindManyDisbursals::WithCreditFacilityId(id),
                sort.into(),
                query,
            )
            .await?;
        Ok(disbursals)
    }

    pub async fn find_disbursal_by_concluded_tx_id(
        &self,
        tx_id: impl Into<crate::primitives::LedgerTxId> + std::fmt::Debug,
    ) -> Result<Disbursal, CoreCreditError> {
        let tx_id = tx_id.into();
        let disbursal = self.disbursals.find_by_concluded_tx_id(Some(tx_id)).await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id(disbursal.facility_id)
            .await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;

        Ok(disbursal)
    }

    pub async fn find_payment_by_id(
        &self,
        payment_id: impl Into<PaymentId> + std::fmt::Debug,
    ) -> Result<Payment, CoreCreditError> {
        let payment = self.payments.find_by_id(payment_id.into()).await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id(payment.credit_facility_id)
            .await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            CoreCreditObject::all_credit_facilities(),
            CoreCreditAction::CREDIT_FACILITY_READ,
        )
        .await?;

        Ok(payment)
    }
}
