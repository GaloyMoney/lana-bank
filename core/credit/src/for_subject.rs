use audit::AuditSvc;
use es_entity::{PaginatedQueryArgs, PaginatedQueryRet};

use super::*;

pub struct CreditFacilitiesForSubject<'a> {
    customer_id: CustomerId,
    subject: &'a Subject,
    authz: &'a Authorization,
    credit_facilities: &'a CreditFacilityRepo,
    disbursals: &'a DisbursalRepo,
    payments: &'a PaymentRepo,
}

impl<'a> CreditFacilitiesForSubject<'a> {
    pub(super) fn new(
        subject: &'a Subject,
        customer_id: CustomerId,
        authz: &'a Authorization,
        credit_facilities: &'a CreditFacilityRepo,
        disbursals: &'a DisbursalRepo,
        payments: &'a PaymentRepo,
    ) -> Self {
        Self {
            customer_id,
            subject,
            authz,
            credit_facilities,
            disbursals,
            payments,
        }
    }

    pub async fn list_credit_facilities_by_created_at(
        &self,
        query: PaginatedQueryArgs<CreditFacilitiesByCreatedAtCursor>,
        direction: ListDirection,
    ) -> Result<
        PaginatedQueryRet<CreditFacility, CreditFacilitiesByCreatedAtCursor>,
        CreditFacilityError,
    > {
        self.authz
            .audit()
            .record_entry(
                self.subject,
                Object::CreditFacility,
                CreditFacilityAction::List,
                true,
            )
            .await?;

        self.credit_facilities
            .list_for_customer_id_by_created_at(self.customer_id, query, direction)
            .await
    }

    pub async fn balance(
        &self,
        id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacilityBalance, CreditFacilityError> {
        let credit_facility = self.credit_facilities.find_by_id(id.into()).await?;

        self.ensure_credit_facility_access(
            &credit_facility,
            Object::CreditFacility,
            CreditFacilityAction::Read,
        )
        .await?;

        Ok(credit_facility.balances())
    }

    pub async fn find_by_id(
        &self,
        id: impl Into<CreditFacilityId>,
    ) -> Result<Option<CreditFacility>, CreditFacilityError> {
        match self.credit_facilities.find_by_id(id.into()).await {
            Ok(cf) => {
                self.ensure_credit_facility_access(
                    &cf,
                    Object::CreditFacility,
                    CreditFacilityAction::Read,
                )
                .await?;
                Ok(Some(cf))
            }
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn ensure_credit_facility_access(
        &self,
        credit_facility: &CreditFacility,
        object: Object,
        action: CreditFacilityAction,
    ) -> Result<(), CreditFacilityError> {
        if credit_facility.customer_id != self.customer_id {
            self.authz
                .audit()
                .record_entry(self.subject, object, action, false)
                .await?;
            return Err(CreditFacilityError::CustomerMismatchForCreditFacility);
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
    ) -> Result<es_entity::PaginatedQueryRet<Disbursal, DisbursalsCursor>, CreditFacilityError>
    {
        let credit_facility = self.credit_facilities.find_by_id(id).await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            Object::CreditFacility,
            CreditFacilityAction::ListDisbursals,
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
    ) -> Result<Disbursal, CreditFacilityError> {
        let tx_id = tx_id.into();
        let disbursal = self.disbursals.find_by_concluded_tx_id(Some(tx_id)).await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id(disbursal.facility_id)
            .await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            Object::CreditFacility,
            CreditFacilityAction::Read,
        )
        .await?;

        Ok(disbursal)
    }

    pub async fn find_payment_by_id(
        &self,
        payment_id: impl Into<PaymentId> + std::fmt::Debug,
    ) -> Result<Payment, CreditFacilityError> {
        let payment = self.payments.find_by_id(payment_id.into()).await?;

        let credit_facility = self
            .credit_facilities
            .find_by_id(payment.facility_id)
            .await?;
        self.ensure_credit_facility_access(
            &credit_facility,
            Object::CreditFacility,
            CreditFacilityAction::Read,
        )
        .await?;

        Ok(payment)
    }
}
