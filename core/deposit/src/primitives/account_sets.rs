use cala_ledger::DebitOrCredit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum DepositAccountCategory {
    Asset,
    Liability,
}

impl From<DepositAccountCategory> for core_accounting::AccountCategory {
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
}

impl DepositSummaryAccountSetSpec {
    pub const fn new(
        name: &'static str,
        external_ref: &'static str,
        account_category: DepositAccountCategory,
        normal_balance_type: DebitOrCredit,
    ) -> Self {
        Self {
            name,
            external_ref,
            account_category,
            normal_balance_type,
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
}

impl DepositOmnibusAccountSetSpec {
    pub const fn new(
        name: &'static str,
        account_set_ref: &'static str,
        account_ref: &'static str,
        account_category: DepositAccountCategory,
        normal_balance_type: DebitOrCredit,
    ) -> Self {
        Self {
            name,
            account_set_ref,
            account_ref,
            account_category,
            normal_balance_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepositAccountSetCatalog {
    deposit: DepositAccountSetCatalogGroup,
    frozen: DepositAccountSetCatalogGroup,
    omnibus: DepositOmnibusAccountSetSpec,
}

#[derive(Debug, Clone)]
pub struct DepositAccountSetCatalogGroup {
    pub individual: DepositSummaryAccountSetSpec,
    pub government_entity: DepositSummaryAccountSetSpec,
    pub private_company: DepositSummaryAccountSetSpec,
    pub bank: DepositSummaryAccountSetSpec,
    pub financial_institution: DepositSummaryAccountSetSpec,
    pub non_domiciled_company: DepositSummaryAccountSetSpec,
}

impl DepositAccountSetCatalog {
    pub fn deposit(&self) -> &DepositAccountSetCatalogGroup {
        &self.deposit
    }

    pub fn frozen(&self) -> &DepositAccountSetCatalogGroup {
        &self.frozen
    }

    pub fn omnibus(&self) -> &DepositOmnibusAccountSetSpec {
        &self.omnibus
    }

    pub fn deposit_specs(&self) -> [DepositSummaryAccountSetSpec; 6] {
        [
            self.deposit.individual,
            self.deposit.government_entity,
            self.deposit.private_company,
            self.deposit.bank,
            self.deposit.financial_institution,
            self.deposit.non_domiciled_company,
        ]
    }

    pub fn frozen_specs(&self) -> [DepositSummaryAccountSetSpec; 6] {
        [
            self.frozen.individual,
            self.frozen.government_entity,
            self.frozen.private_company,
            self.frozen.bank,
            self.frozen.financial_institution,
            self.frozen.non_domiciled_company,
        ]
    }

    pub fn omnibus_specs(&self) -> [DepositOmnibusAccountSetSpec; 1] {
        [self.omnibus]
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
    );

const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME: &str = "Deposit Government Entity Account Set";
const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF: &str = "deposit-government-entity-account-set";
const DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_NAME,
        DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
    );

const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME: &str = "Deposit Private Company Account Set";
const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF: &str = "deposit-private-company-account-set";
const DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_NAME,
        DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
    );

const DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Deposit Bank Account Set";
const DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "deposit-bank-account-set";
const DEPOSIT_BANK_ACCOUNT_SET: DepositSummaryAccountSetSpec = DepositSummaryAccountSetSpec::new(
    DEPOSIT_BANK_ACCOUNT_SET_NAME,
    DEPOSIT_BANK_ACCOUNT_SET_REF,
    DepositAccountCategory::Liability,
    DebitOrCredit::Credit,
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
    );

const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME: &str = "Frozen Deposit Individual Account Set";
const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF: &str = "frozen-deposit-individual-account-set";
const FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
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
    );

const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME: &str = "Frozen Deposit Bank Account Set";
const FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF: &str = "frozen-deposit-bank-account-set";
const FROZEN_DEPOSIT_BANK_ACCOUNT_SET: DepositSummaryAccountSetSpec =
    DepositSummaryAccountSetSpec::new(
        FROZEN_DEPOSIT_BANK_ACCOUNT_SET_NAME,
        FROZEN_DEPOSIT_BANK_ACCOUNT_SET_REF,
        DepositAccountCategory::Liability,
        DebitOrCredit::Credit,
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
);

pub const DEPOSIT_ACCOUNT_SET_CATALOG: DepositAccountSetCatalog = DepositAccountSetCatalog {
    deposit: DepositAccountSetCatalogGroup {
        individual: DEPOSIT_INDIVIDUAL_ACCOUNT_SET,
        government_entity: DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET,
        private_company: DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET,
        bank: DEPOSIT_BANK_ACCOUNT_SET,
        financial_institution: DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET,
        non_domiciled_company: DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET,
    },
    frozen: DepositAccountSetCatalogGroup {
        individual: FROZEN_DEPOSIT_INDIVIDUAL_ACCOUNT_SET,
        government_entity: FROZEN_DEPOSIT_GOVERNMENT_ENTITY_ACCOUNT_SET,
        private_company: FROZEN_DEPOSIT_PRIVATE_COMPANY_ACCOUNT_SET,
        bank: FROZEN_DEPOSIT_BANK_ACCOUNT_SET,
        financial_institution: FROZEN_DEPOSIT_FINANCIAL_INSTITUTION_ACCOUNT_SET,
        non_domiciled_company: FROZEN_DEPOSIT_NON_DOMICILED_COMPANY_ACCOUNT_SET,
    },
    omnibus: DEPOSIT_OMNIBUS_ACCOUNT_SET,
};
