use std::sync::LazyLock;

use cala_ledger::DebitOrCredit;
use money::{CurrencyCode, CurrencyMap};

use super::DepositAccountType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum DepositAccountCategory {
    Asset,
    Liability,
}

impl From<DepositAccountCategory> for chart_primitives::AccountCategory {
    fn from(value: DepositAccountCategory) -> Self {
        match value {
            DepositAccountCategory::Asset => Self::Asset,
            DepositAccountCategory::Liability => Self::Liability,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepositSummaryAccountSetSpec {
    pub name: &'static str,
    pub external_ref: &'static str,
    pub account_category: DepositAccountCategory,
    pub normal_balance_type: DebitOrCredit,
    pub currency: CurrencyCode,
}

impl DepositSummaryAccountSetSpec {
    pub const fn new(
        name: &'static str,
        external_ref: &'static str,
        account_category: DepositAccountCategory,
        normal_balance_type: DebitOrCredit,
        currency: CurrencyCode,
    ) -> Self {
        Self {
            name,
            external_ref,
            account_category,
            normal_balance_type,
            currency,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepositOmnibusAccountSetSpec {
    pub name: &'static str,
    pub account_set_ref: &'static str,
    pub account_ref: &'static str,
    pub account_category: DepositAccountCategory,
    pub normal_balance_type: DebitOrCredit,
    pub currency: CurrencyCode,
}

impl DepositOmnibusAccountSetSpec {
    pub const fn new(
        name: &'static str,
        account_set_ref: &'static str,
        account_ref: &'static str,
        account_category: DepositAccountCategory,
        normal_balance_type: DebitOrCredit,
        currency: CurrencyCode,
    ) -> Self {
        Self {
            name,
            account_set_ref,
            account_ref,
            account_category,
            normal_balance_type,
            currency,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepositAccountSetCatalog {
    deposit: DepositAccountSetCatalogGroup,
    frozen: DepositAccountSetCatalogGroup,
    omnibus: CurrencyMap<DepositOmnibusAccountSetSpec>,
}

#[derive(Debug, Clone)]
pub struct DepositAccountSetCatalogGroup {
    pub individual: CurrencyMap<DepositSummaryAccountSetSpec>,
    pub government_entity: CurrencyMap<DepositSummaryAccountSetSpec>,
    pub private_company: CurrencyMap<DepositSummaryAccountSetSpec>,
    pub bank: CurrencyMap<DepositSummaryAccountSetSpec>,
    pub financial_institution: CurrencyMap<DepositSummaryAccountSetSpec>,
    pub non_domiciled_company: CurrencyMap<DepositSummaryAccountSetSpec>,
}

impl DepositAccountSetCatalogGroup {
    pub fn for_type(
        &self,
        deposit_account_type: DepositAccountType,
    ) -> &CurrencyMap<DepositSummaryAccountSetSpec> {
        match deposit_account_type {
            DepositAccountType::Individual => &self.individual,
            DepositAccountType::GovernmentEntity => &self.government_entity,
            DepositAccountType::PrivateCompany => &self.private_company,
            DepositAccountType::Bank => &self.bank,
            DepositAccountType::FinancialInstitution => &self.financial_institution,
            DepositAccountType::NonDomiciledCompany => &self.non_domiciled_company,
        }
    }

    pub fn find(
        &self,
        deposit_account_type: DepositAccountType,
        currency: CurrencyCode,
    ) -> Option<DepositSummaryAccountSetSpec> {
        self.for_type(deposit_account_type).get(&currency).copied()
    }
}

impl DepositAccountSetCatalog {
    pub fn deposit(&self) -> &DepositAccountSetCatalogGroup {
        &self.deposit
    }

    pub fn frozen(&self) -> &DepositAccountSetCatalogGroup {
        &self.frozen
    }

    pub fn omnibus(&self) -> &CurrencyMap<DepositOmnibusAccountSetSpec> {
        &self.omnibus
    }

    pub fn find_omnibus(&self, currency: CurrencyCode) -> Option<DepositOmnibusAccountSetSpec> {
        self.omnibus.get(&currency).copied()
    }

    pub fn deposit_specs(&self) -> Vec<DepositSummaryAccountSetSpec> {
        let mut specs = Vec::new();
        for specs_for_type in [
            &self.deposit.individual,
            &self.deposit.government_entity,
            &self.deposit.private_company,
            &self.deposit.bank,
            &self.deposit.financial_institution,
            &self.deposit.non_domiciled_company,
        ] {
            specs.extend(specs_for_type.values().copied());
        }
        specs
    }

    pub fn frozen_specs(&self) -> Vec<DepositSummaryAccountSetSpec> {
        let mut specs = Vec::new();
        for specs_for_type in [
            &self.frozen.individual,
            &self.frozen.government_entity,
            &self.frozen.private_company,
            &self.frozen.bank,
            &self.frozen.financial_institution,
            &self.frozen.non_domiciled_company,
        ] {
            specs.extend(specs_for_type.values().copied());
        }
        specs
    }

    pub fn omnibus_specs(&self) -> Vec<DepositOmnibusAccountSetSpec> {
        self.omnibus.values().copied().collect()
    }
}

const DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME: &str = "Deposit Individual Account Set";
const DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF: &str = "deposit-individual-account-set";
const DEPOSIT_INDIVIDUAL_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME,
        DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME: &str = "Deposit Government Entity Account Set";
const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF: &str = "deposit-government-entity-account-set";
const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME,
        DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME: &str = "Deposit Private Company Account Set";
const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF: &str = "deposit-private-company-account-set";
const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME,
        DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Deposit Bank Account Set";
const DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "deposit-bank-account-set";
const DEPOSIT_BANK_ACCOUNT_SET: DepositSummaryAccountSetSpec = DepositSummaryAccountSetSpec::new(
    DEPOSIT_BANK_ACCOUNT_SET_NAME,
    DEPOSIT_BANK_ACCOUNT_SET_REF,
    DepositAccountCategory::Liability,
    DebitOrCredit::Credit,
    CurrencyCode::USD,
);

const DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME: &str =
    "Deposit Financial Institution Account Set";
const DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF: &str =
    "deposit-financial-institution-account-set";
const DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME,
        DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_NAME: &str =
    "Deposit Non-Domiciled Company Account Set";
const DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_REF: &str =
    "deposit-non-domiciled-company-account-set";
const DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_NAME,
        DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME: &str = "Frozen Deposit Individual Account Set";
const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF: &str = "frozen-deposit-individual-account-set";
const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Government Entity Account Set";
const FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF: &str =
    "frozen-deposit-government-entity-account-set";
const FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Private Company Account Set";
const FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF: &str =
    "frozen-deposit-private-company-account-set";
const FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Frozen Deposit Bank Account Set";
const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "frozen-deposit-bank-account-set";
const FROZEN_DEPOSIT_BANK_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Financial Institution Account Set";
const FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF: &str =
    "frozen-deposit-financial-institution-account-set";
const FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_NAME: &str =
    "Frozen Deposit Non-Domiciled Company Account Set";
const FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_REF: &str =
    "frozen-deposit-non-domiciled-company-account-set";
const FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
        CurrencyCode::USD,
    );

const DEPOSIT_OMNIBUS_ACCOUNT_SET_NAME: &str = "Deposit Omnibus Account Set";
const DEPOSIT_OMNIBUS_ACCOUNT_SET_REF: &str = "deposit-omnibus-account-set";
const DEPOSIT_OMNIBUS_ACCOUNT_REF: &str = "deposit-omnibus-account";
const DEPOSIT_OMNIBUS_ACCOUNT_SET: DepositOmnibusAccountSetSpec = DepositOmnibusAccountSetSpec::new(
    DEPOSIT_OMNIBUS_ACCOUNT_SET_NAME,
    DEPOSIT_OMNIBUS_ACCOUNT_SET_REF,
    DEPOSIT_OMNIBUS_ACCOUNT_REF,
    DepositAccountCategory::Asset,
    DebitOrCredit::Debit,
    CurrencyCode::USD,
);

fn summary_currency_map(
    specs: impl IntoIterator<Item = DepositSummaryAccountSetSpec>,
) -> CurrencyMap<DepositSummaryAccountSetSpec> {
    specs
        .into_iter()
        .map(|spec| (spec.currency, spec))
        .collect()
}

fn omnibus_currency_map(
    specs: impl IntoIterator<Item = DepositOmnibusAccountSetSpec>,
) -> CurrencyMap<DepositOmnibusAccountSetSpec> {
    specs
        .into_iter()
        .map(|spec| (spec.currency, spec))
        .collect()
}

pub static DEPOSIT_ACCOUNT_SET_CATALOG: LazyLock<DepositAccountSetCatalog> =
    LazyLock::new(|| DepositAccountSetCatalog {
        deposit: DepositAccountSetCatalogGroup {
            individual: summary_currency_map([DEPOSIT_INDIVIDUAL_ACCOUNT_SET]),
            government_entity: summary_currency_map([DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET]),
            private_company: summary_currency_map([DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET]),
            bank: summary_currency_map([DEPOSIT_BANK_ACCOUNT_SET]),
            financial_institution: summary_currency_map([
                DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET,
            ]),
            non_domiciled_company: summary_currency_map([
                DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET,
            ]),
        },
        frozen: DepositAccountSetCatalogGroup {
            individual: summary_currency_map([FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET]),
            government_entity: summary_currency_map([FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET]),
            private_company: summary_currency_map([FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET]),
            bank: summary_currency_map([FROZEN_DEPOSIT_BANK_ACCOUNT_SET]),
            financial_institution: summary_currency_map([
                FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET,
            ]),
            non_domiciled_company: summary_currency_map([
                FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET,
            ]),
        },
        omnibus: omnibus_currency_map([DEPOSIT_OMNIBUS_ACCOUNT_SET]),
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_catalog_finds_configured_currency_for_account_type() {
        let spec = DEPOSIT_ACCOUNT_SET_CATALOG
            .deposit()
            .find(DepositAccountType::Individual, CurrencyCode::USD);

        let spec = spec.expect("expected USD deposit spec for individual accounts");
        assert_eq!(
            spec.external_ref,
            DEPOSIT_INDIVIDUAL_ACCOUNT_SET.external_ref
        );
        assert_eq!(spec.currency, CurrencyCode::USD);
        assert!(
            DEPOSIT_ACCOUNT_SET_CATALOG
                .deposit()
                .find(DepositAccountType::Individual, CurrencyCode::BTC)
                .is_none()
        );
    }

    #[test]
    fn omnibus_catalog_finds_configured_currency() {
        let spec = DEPOSIT_ACCOUNT_SET_CATALOG
            .find_omnibus(CurrencyCode::USD)
            .expect("expected USD omnibus spec");

        assert_eq!(
            spec.account_set_ref,
            DEPOSIT_OMNIBUS_ACCOUNT_SET.account_set_ref
        );
        assert_eq!(spec.currency, CurrencyCode::USD);
        assert!(
            DEPOSIT_ACCOUNT_SET_CATALOG
                .find_omnibus(CurrencyCode::BTC)
                .is_none()
        );
    }
}
