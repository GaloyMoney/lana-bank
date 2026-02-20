use serde::{Deserialize, Serialize};

use cala_ledger_core_types::primitives::DebitOrCredit;

mod balance;
mod credit_facility_accounts;
mod disbursal_accounts;
pub mod error;
mod trait_def;

pub use trait_def::CreditLedgerOps;

use crate::{collateral::ledger::CollateralAccountSets, primitives::CalaAccountSetId};

pub use balance::*;
pub use credit_facility_accounts::*;
pub use disbursal_accounts::*;
pub use error::*;

#[derive(Clone, Copy)]
pub struct InternalAccountSetDetails {
    pub id: CalaAccountSetId,
    pub normal_balance_type: DebitOrCredit,
}

impl InternalAccountSetDetails {
    pub fn id(&self) -> CalaAccountSetId {
        self.id
    }

    pub fn normal_balance_type(&self) -> DebitOrCredit {
        self.normal_balance_type
    }
}

#[derive(Clone, Copy)]
pub struct DisbursedReceivableAccountSets {
    pub individual: InternalAccountSetDetails,
    pub government_entity: InternalAccountSetDetails,
    pub private_company: InternalAccountSetDetails,
    pub bank: InternalAccountSetDetails,
    pub financial_institution: InternalAccountSetDetails,
    pub foreign_agency_or_subsidiary: InternalAccountSetDetails,
    pub non_domiciled_company: InternalAccountSetDetails,
}

#[derive(Clone, Copy)]
pub struct DisbursedReceivable {
    pub short_term: DisbursedReceivableAccountSets,
    pub long_term: DisbursedReceivableAccountSets,
    pub overdue: DisbursedReceivableAccountSets,
}

#[derive(Clone, Copy)]
pub struct InterestReceivableAccountSets {
    pub individual: InternalAccountSetDetails,
    pub government_entity: InternalAccountSetDetails,
    pub private_company: InternalAccountSetDetails,
    pub bank: InternalAccountSetDetails,
    pub financial_institution: InternalAccountSetDetails,
    pub foreign_agency_or_subsidiary: InternalAccountSetDetails,
    pub non_domiciled_company: InternalAccountSetDetails,
}

#[derive(Clone, Copy)]
pub struct InterestReceivable {
    pub short_term: InterestReceivableAccountSets,
    pub long_term: InterestReceivableAccountSets,
}

#[derive(Clone, Copy)]
pub struct CreditFacilityInternalAccountSets {
    pub facility: InternalAccountSetDetails,
    pub collateral: CollateralAccountSets,
    pub proceeds_from_liquidation: InternalAccountSetDetails,
    pub disbursed_receivable: DisbursedReceivable,
    pub disbursed_defaulted: InternalAccountSetDetails,
    pub interest_receivable: InterestReceivable,
    pub interest_defaulted: InternalAccountSetDetails,
    pub interest_income: InternalAccountSetDetails,
    pub fee_income: InternalAccountSetDetails,
    pub uncovered_outstanding: InternalAccountSetDetails,
    pub payment_holding: InternalAccountSetDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShortTermDisbursedIntegrationMeta {
    pub short_term_individual_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_government_entity_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_private_company_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_bank_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_financial_institution_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LongTermDisbursedIntegrationMeta {
    pub long_term_individual_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_government_entity_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_private_company_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_bank_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_financial_institution_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShortTermInterestIntegrationMeta {
    pub short_term_individual_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_government_entity_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_private_company_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_bank_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub short_term_financial_institution_interest_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub short_term_non_domiciled_company_interest_receivable_parent_account_set_id:
        CalaAccountSetId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LongTermInterestIntegrationMeta {
    pub long_term_individual_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_government_entity_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_private_company_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_bank_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_financial_institution_interest_receivable_parent_account_set_id: CalaAccountSetId,
    pub long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub long_term_non_domiciled_company_interest_receivable_parent_account_set_id: CalaAccountSetId,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OverdueDisbursedIntegrationMeta {
    pub overdue_individual_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub overdue_government_entity_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub overdue_private_company_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub overdue_bank_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub overdue_financial_institution_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
    pub overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
        CalaAccountSetId,
    pub overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id: CalaAccountSetId,
}
