#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum CreditAccountCategory {
    OffBalanceSheet,
    Asset,
    Liability,
    Equity,
    Revenue,
    CostOfRevenue,
    Expenses,
}

impl From<CreditAccountCategory> for core_accounting::AccountCategory {
    fn from(value: CreditAccountCategory) -> Self {
        match value {
            CreditAccountCategory::OffBalanceSheet => Self::OffBalanceSheet,
            CreditAccountCategory::Asset => Self::Asset,
            CreditAccountCategory::Liability => Self::Liability,
            CreditAccountCategory::Equity => Self::Equity,
            CreditAccountCategory::Revenue => Self::Revenue,
            CreditAccountCategory::CostOfRevenue => Self::CostOfRevenue,
            CreditAccountCategory::Expenses => Self::Expenses,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CreditOmnibusAccountSetSpec {
    pub name: &'static str,
    pub account_set_ref: &'static str,
    pub account_ref: &'static str,
    pub account_category: CreditAccountCategory,
}

impl CreditOmnibusAccountSetSpec {
    pub const fn new(
        name: &'static str,
        account_set_ref: &'static str,
        account_ref: &'static str,
        account_category: CreditAccountCategory,
    ) -> Self {
        Self {
            name,
            account_set_ref,
            account_ref,
            account_category,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CreditSummaryAccountSetSpec {
    pub name: &'static str,
    pub external_ref: &'static str,
    pub account_category: CreditAccountCategory,
}

impl CreditSummaryAccountSetSpec {
    pub const fn new(
        name: &'static str,
        external_ref: &'static str,
        account_category: CreditAccountCategory,
    ) -> Self {
        Self {
            name,
            external_ref,
            account_category,
        }
    }
}

// Omnibus Accounts
const CREDIT_COLLATERAL_OMNIBUS_NAME: &str = "Credit Collateral Omnibus Account Set";
const CREDIT_COLLATERAL_OMNIBUS_SET_REF: &str = "credit-collateral-omnibus-account-set";
const CREDIT_COLLATERAL_OMNIBUS_ACCOUNT_REF: &str = "credit-collateral-omnibus-account";
pub const CREDIT_COLLATERAL_OMNIBUS: CreditOmnibusAccountSetSpec = CreditOmnibusAccountSetSpec::new(
    CREDIT_COLLATERAL_OMNIBUS_NAME,
    CREDIT_COLLATERAL_OMNIBUS_SET_REF,
    CREDIT_COLLATERAL_OMNIBUS_ACCOUNT_REF,
    CreditAccountCategory::OffBalanceSheet,
);

const CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_NAME: &str =
    "Credit Interest Added to Obligations Omnibus Account Set";
const CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_SET_REF: &str =
    "credit-interest-added-to-obligations-omnibus-account-set";
const CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_ACCOUNT_REF: &str =
    "credit-interest-added-to-obligations-omnibus-account";
pub const CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS: CreditOmnibusAccountSetSpec =
    CreditOmnibusAccountSetSpec::new(
        CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_NAME,
        CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_SET_REF,
        CREDIT_INTEREST_ADDED_TO_OBLIGATIONS_OMNIBUS_ACCOUNT_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const CREDIT_PAYMENTS_MADE_OMNIBUS_NAME: &str = "Credit Payments Made Omnibus Account Set";
const CREDIT_PAYMENTS_MADE_OMNIBUS_SET_REF: &str = "credit-payments-made-omnibus-account-set";
const CREDIT_PAYMENTS_MADE_OMNIBUS_ACCOUNT_REF: &str = "credit-payments-made-omnibus-account";
pub const CREDIT_PAYMENTS_MADE_OMNIBUS: CreditOmnibusAccountSetSpec =
    CreditOmnibusAccountSetSpec::new(
        CREDIT_PAYMENTS_MADE_OMNIBUS_NAME,
        CREDIT_PAYMENTS_MADE_OMNIBUS_SET_REF,
        CREDIT_PAYMENTS_MADE_OMNIBUS_ACCOUNT_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const CREDIT_FACILITY_OMNIBUS_NAME: &str = "Credit Facility Omnibus Account Set";
const CREDIT_FACILITY_OMNIBUS_SET_REF: &str = "credit-facility-omnibus-account-set";
const CREDIT_FACILITY_OMNIBUS_ACCOUNT_REF: &str = "credit-facility-omnibus-account";
pub const CREDIT_FACILITY_OMNIBUS: CreditOmnibusAccountSetSpec = CreditOmnibusAccountSetSpec::new(
    CREDIT_FACILITY_OMNIBUS_NAME,
    CREDIT_FACILITY_OMNIBUS_SET_REF,
    CREDIT_FACILITY_OMNIBUS_ACCOUNT_REF,
    CreditAccountCategory::OffBalanceSheet,
);

const CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_NAME: &str =
    "Credit Facility Liquidation Proceeds Omnibus Account Set";
const CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_SET_REF: &str =
    "credit-facility-liquidation-proceeds-omnibus-account-set";
const CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_ACCOUNT_REF: &str =
    "credit-facility-liquidation-proceeds-omnibus-account";
pub const CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS: CreditOmnibusAccountSetSpec =
    CreditOmnibusAccountSetSpec::new(
        CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_NAME,
        CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_SET_REF,
        CREDIT_FACILITY_LIQUIDATION_PROCEEDS_OMNIBUS_ACCOUNT_REF,
        CreditAccountCategory::Revenue,
    );

// Summary Accounts
const CREDIT_FACILITY_REMAINING_NAME: &str = "Credit Facility Remaining Account Set";
const CREDIT_FACILITY_REMAINING_REF: &str = "credit-facility-remaining-account-set";
pub const CREDIT_FACILITY_REMAINING: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_FACILITY_REMAINING_NAME,
    CREDIT_FACILITY_REMAINING_REF,
    CreditAccountCategory::OffBalanceSheet,
);

const CREDIT_COLLATERAL_NAME: &str = "Credit Collateral Account Set";
const CREDIT_COLLATERAL_REF: &str = "credit-collateral-account-set";
pub const CREDIT_COLLATERAL: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_COLLATERAL_NAME,
    CREDIT_COLLATERAL_REF,
    CreditAccountCategory::OffBalanceSheet,
);

const CREDIT_FACILITY_COLLATERAL_IN_LIQUIDATION_NAME: &str =
    "Credit Facility Collateral In-Liquidation Account Set";
const CREDIT_FACILITY_COLLATERAL_IN_LIQUIDATION_REF: &str =
    "credit-facility-collateral-in-liquidation-account-set";
pub const CREDIT_FACILITY_COLLATERAL_IN_LIQUIDATION: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        CREDIT_FACILITY_COLLATERAL_IN_LIQUIDATION_NAME,
        CREDIT_FACILITY_COLLATERAL_IN_LIQUIDATION_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const CREDIT_FACILITY_LIQUIDATED_COLLATERAL_NAME: &str =
    "Credit Facility Liquidated Collateral Account Set";
const CREDIT_FACILITY_LIQUIDATED_COLLATERAL_REF: &str =
    "credit-facility-liquidated-collateral-account-set";
pub const CREDIT_FACILITY_LIQUIDATED_COLLATERAL: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        CREDIT_FACILITY_LIQUIDATED_COLLATERAL_NAME,
        CREDIT_FACILITY_LIQUIDATED_COLLATERAL_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const CREDIT_FACILITY_PROCEEDS_FROM_LIQUIDATION_NAME: &str =
    "Credit Facility Proceeds From Liquidation Account Set";
const CREDIT_FACILITY_PROCEEDS_FROM_LIQUIDATION_REF: &str =
    "credit-facility-proceeds-from-liquidation-account-set";
pub const CREDIT_FACILITY_PROCEEDS_FROM_LIQUIDATION: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        CREDIT_FACILITY_PROCEEDS_FROM_LIQUIDATION_NAME,
        CREDIT_FACILITY_PROCEEDS_FROM_LIQUIDATION_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const SHORT_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Individual Disbursed Receivable Account Set";
const SHORT_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-individual-disbursed-receivable-account-set";
pub const SHORT_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Government Entity Disbursed Receivable Account Set";
const SHORT_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-government-entity-disbursed-receivable-account-set";
pub const SHORT_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Private Company Disbursed Receivable Account Set";
const SHORT_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-private-company-disbursed-receivable-account-set";
pub const SHORT_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_BANK_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Bank Disbursed Receivable Account Set";
const SHORT_TERM_BANK_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-bank-disbursed-receivable-account-set";
pub const SHORT_TERM_BANK_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_BANK_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_BANK_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Financial Institution Disbursed Receivable Account Set";
const SHORT_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-financial-institution-disbursed-receivable-account-set";
pub const SHORT_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Foreign Agency Or Subsidiary Disbursed Receivable Account Set";
const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-foreign-agency-or-subsidiary-disbursed-receivable-account-set";
pub const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE:
    CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME,
    SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF,
    CreditAccountCategory::Asset,
);

const SHORT_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Short Term Credit Non-Domiciled Company Disbursed Receivable Account Set";
const SHORT_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "short-term-credit-non-domiciled-company-disbursed-receivable-account-set";
pub const SHORT_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME,
        SHORT_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Individual Disbursed Receivable Account Set";
const LONG_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-individual-disbursed-receivable-account-set";
pub const LONG_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_INDIVIDUAL_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Government Entity Disbursed Receivable Account Set";
const LONG_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-government-entity-disbursed-receivable-account-set";
pub const LONG_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Private Company Disbursed Receivable Account Set";
const LONG_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-private-company-disbursed-receivable-account-set";
pub const LONG_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_BANK_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Bank Disbursed Receivable Account Set";
const LONG_TERM_BANK_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-bank-disbursed-receivable-account-set";
pub const LONG_TERM_BANK_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_BANK_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_BANK_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Financial Institution Disbursed Receivable Account Set";
const LONG_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-financial-institution-disbursed-receivable-account-set";
pub const LONG_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Foreign Agency Or Subsidiary Disbursed Receivable Account Set";
const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-foreign-agency-or-subsidiary-disbursed-receivable-account-set";
pub const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Long Term Credit Non-Domiciled Company Disbursed Receivable Account Set";
const LONG_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "long-term-credit-non-domiciled-company-disbursed-receivable-account-set";
pub const LONG_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME,
        LONG_TERM_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Individual Interest Receivable Account Set";
const SHORT_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-individual-interest-receivable-account-set";
pub const SHORT_TERM_INDIVIDUAL_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Government Entity Interest Receivable Account Set";
const SHORT_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-government-entity-interest-receivable-account-set";
pub const SHORT_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Private Company Interest Receivable Account Set";
const SHORT_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-private-company-interest-receivable-account-set";
pub const SHORT_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_BANK_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Bank Interest Receivable Account Set";
const SHORT_TERM_BANK_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-bank-interest-receivable-account-set";
pub const SHORT_TERM_BANK_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_BANK_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_BANK_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Financial Institution Interest Receivable Account Set";
const SHORT_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-financial-institution-interest-receivable-account-set";
pub const SHORT_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Foreign Agency Or Subsidiary Interest Receivable Account Set";
const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-foreign-agency-or-subsidiary-interest-receivable-account-set";
pub const SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const SHORT_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_NAME: &str =
    "Short Term Credit Non-Domiciled Company Interest Receivable Account Set";
const SHORT_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_REF: &str =
    "short-term-credit-non-domiciled-company-interest-receivable-account-set";
pub const SHORT_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        SHORT_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_NAME,
        SHORT_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Individual Interest Receivable Account Set";
const LONG_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-individual-interest-receivable-account-set";
pub const LONG_TERM_INDIVIDUAL_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_INDIVIDUAL_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Government Entity Interest Receivable Account Set";
const LONG_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-government-entity-interest-receivable-account-set";
pub const LONG_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_GOVERNMENT_ENTITY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Private Company Interest Receivable Account Set";
const LONG_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-private-company-interest-receivable-account-set";
pub const LONG_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_PRIVATE_COMPANY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_BANK_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Bank Interest Receivable Account Set";
const LONG_TERM_BANK_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-bank-interest-receivable-account-set";
pub const LONG_TERM_BANK_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_BANK_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_BANK_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Financial Institution Interest Receivable Account Set";
const LONG_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-financial-institution-interest-receivable-account-set";
pub const LONG_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_FINANCIAL_INSTITUTION_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Foreign Agency Or Subsidiary Interest Receivable Account Set";
const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-foreign-agency-or-subsidiary-interest-receivable-account-set";
pub const LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_FOREIGN_AGENCY_OR_SUBSIDIARY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const LONG_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_NAME: &str =
    "Long Term Credit Non-Domiciled Company Interest Receivable Account Set";
const LONG_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_REF: &str =
    "long-term-credit-non-domiciled-company-interest-receivable-account-set";
pub const LONG_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        LONG_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_NAME,
        LONG_TERM_NON_DOMICILED_COMPANY_INTEREST_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Individual Disbursed Receivable Account Set";
const OVERDUE_INDIVIDUAL_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-individual-disbursed-receivable-account-set";
pub const OVERDUE_INDIVIDUAL_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_INDIVIDUAL_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_INDIVIDUAL_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Government Entity Disbursed Receivable Account Set";
const OVERDUE_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-government-entity-disbursed-receivable-account-set";
pub const OVERDUE_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_GOVERNMENT_ENTITY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Private Company Disbursed Receivable Account Set";
const OVERDUE_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-private-company-disbursed-receivable-account-set";
pub const OVERDUE_PRIVATE_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_PRIVATE_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_BANK_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Bank Disbursed Receivable Account Set";
const OVERDUE_BANK_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-bank-disbursed-receivable-account-set";
pub const OVERDUE_BANK_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_BANK_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_BANK_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Financial Institution Disbursed Receivable Account Set";
const OVERDUE_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-financial-institution-disbursed-receivable-account-set";
pub const OVERDUE_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_FINANCIAL_INSTITUTION_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Foreign Agency Or Subsidiary Disbursed Receivable Account Set";
const OVERDUE_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-foreign-agency-or-subsidiary-disbursed-receivable-account-set";
pub const OVERDUE_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_FOREIGN_AGENCY_OR_SUBSIDIARY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const OVERDUE_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME: &str =
    "Overdue Credit Non-Domiciled Company Disbursed Receivable Account Set";
const OVERDUE_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF: &str =
    "overdue-credit-non-domiciled-company-disbursed-receivable-account-set";
pub const OVERDUE_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        OVERDUE_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_NAME,
        OVERDUE_NON_DOMICILED_COMPANY_DISBURSED_RECEIVABLE_REF,
        CreditAccountCategory::Asset,
    );

const CREDIT_DISBURSED_DEFAULTED_NAME: &str = "Credit Disbursed Defaulted Account Set";
const CREDIT_DISBURSED_DEFAULTED_REF: &str = "credit-disbursed-defaulted-account-set";
pub const CREDIT_DISBURSED_DEFAULTED: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        CREDIT_DISBURSED_DEFAULTED_NAME,
        CREDIT_DISBURSED_DEFAULTED_REF,
        CreditAccountCategory::Asset,
    );

const CREDIT_INTEREST_DEFAULTED_NAME: &str = "Credit Interest Defaulted Account Set";
const CREDIT_INTEREST_DEFAULTED_REF: &str = "credit-interest-defaulted-account-set";
pub const CREDIT_INTEREST_DEFAULTED: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_INTEREST_DEFAULTED_NAME,
    CREDIT_INTEREST_DEFAULTED_REF,
    CreditAccountCategory::Asset,
);

const CREDIT_INTEREST_INCOME_NAME: &str = "Credit Interest Income Account Set";
const CREDIT_INTEREST_INCOME_REF: &str = "credit-interest-income-account-set";
pub const CREDIT_INTEREST_INCOME: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_INTEREST_INCOME_NAME,
    CREDIT_INTEREST_INCOME_REF,
    CreditAccountCategory::Revenue,
);

const CREDIT_FEE_INCOME_NAME: &str = "Credit Fee Income Account Set";
const CREDIT_FEE_INCOME_REF: &str = "credit-fee-income-account-set";
pub const CREDIT_FEE_INCOME: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_FEE_INCOME_NAME,
    CREDIT_FEE_INCOME_REF,
    CreditAccountCategory::Revenue,
);

const CREDIT_UNCOVERED_OUTSTANDING_NAME: &str = "Credit Uncovered Outstanding Account Set";
const CREDIT_UNCOVERED_OUTSTANDING_REF: &str = "credit-unconvered-outstanding-account-set";
pub const CREDIT_UNCOVERED_OUTSTANDING: CreditSummaryAccountSetSpec =
    CreditSummaryAccountSetSpec::new(
        CREDIT_UNCOVERED_OUTSTANDING_NAME,
        CREDIT_UNCOVERED_OUTSTANDING_REF,
        CreditAccountCategory::OffBalanceSheet,
    );

const CREDIT_PAYMENT_HOLDING_NAME: &str = "Credit Payment Holding Account Set";
const CREDIT_PAYMENT_HOLDING_REF: &str = "credit-payment-holding-account-set";
pub const CREDIT_PAYMENT_HOLDING: CreditSummaryAccountSetSpec = CreditSummaryAccountSetSpec::new(
    CREDIT_PAYMENT_HOLDING_NAME,
    CREDIT_PAYMENT_HOLDING_REF,
    CreditAccountCategory::Asset,
);
