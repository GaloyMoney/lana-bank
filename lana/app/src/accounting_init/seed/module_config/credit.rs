use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    credit::{ChartOfAccountsIntegrationConfig, Credit},
};

use rbac_types::Subject;

#[derive(Deserialize)]
struct CreditConfigData {
    facility_omnibus_parent_code: String,
    collateral_omnibus_parent_code: String,
    liquidation_proceeds_omnibus_parent_code: String,
    payments_made_omnibus_parent_code: String,
    interest_added_to_obligations_omnibus_parent_code: String,
    facility_parent_code: String,
    collateral_parent_code: String,
    collateral_in_liquidation_parent_code: String,
    interest_income_parent_code: String,
    fee_income_parent_code: String,
    payment_holding_parent_code: String,
    uncovered_outstanding_parent_code: String,
    short_term_individual_interest_receivable_parent_code: String,
    short_term_government_entity_interest_receivable_parent_code: String,
    short_term_private_company_interest_receivable_parent_code: String,
    short_term_bank_interest_receivable_parent_code: String,
    short_term_financial_institution_interest_receivable_parent_code: String,
    short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code: String,
    short_term_non_domiciled_company_interest_receivable_parent_code: String,
    long_term_individual_interest_receivable_parent_code: String,
    long_term_government_entity_interest_receivable_parent_code: String,
    long_term_private_company_interest_receivable_parent_code: String,
    long_term_bank_interest_receivable_parent_code: String,
    long_term_financial_institution_interest_receivable_parent_code: String,
    long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code: String,
    long_term_non_domiciled_company_interest_receivable_parent_code: String,
    short_term_individual_disbursed_receivable_parent_code: String,
    short_term_government_entity_disbursed_receivable_parent_code: String,
    short_term_private_company_disbursed_receivable_parent_code: String,
    short_term_bank_disbursed_receivable_parent_code: String,
    short_term_financial_institution_disbursed_receivable_parent_code: String,
    short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code: String,
    short_term_non_domiciled_company_disbursed_receivable_parent_code: String,
    long_term_individual_disbursed_receivable_parent_code: String,
    long_term_government_entity_disbursed_receivable_parent_code: String,
    long_term_private_company_disbursed_receivable_parent_code: String,
    long_term_bank_disbursed_receivable_parent_code: String,
    long_term_financial_institution_disbursed_receivable_parent_code: String,
    long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code: String,
    long_term_non_domiciled_company_disbursed_receivable_parent_code: String,
    overdue_individual_disbursed_receivable_parent_code: String,
    overdue_government_entity_disbursed_receivable_parent_code: String,
    overdue_private_company_disbursed_receivable_parent_code: String,
    overdue_bank_disbursed_receivable_parent_code: String,
    overdue_financial_institution_disbursed_receivable_parent_code: String,
    overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code: String,
    overdue_non_domiciled_company_disbursed_receivable_parent_code: String,
}

pub(in crate::accounting_init::seed) async fn credit_module_configure(
    credit: &Credit,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let CreditConfigData {
        facility_omnibus_parent_code,
        collateral_omnibus_parent_code,
        liquidation_proceeds_omnibus_parent_code,
        payments_made_omnibus_parent_code,
        interest_added_to_obligations_omnibus_parent_code,
        facility_parent_code,
        collateral_parent_code,
        collateral_in_liquidation_parent_code,
        interest_income_parent_code,
        fee_income_parent_code,
        payment_holding_parent_code,
        uncovered_outstanding_parent_code,
        short_term_individual_interest_receivable_parent_code,
        short_term_government_entity_interest_receivable_parent_code,
        short_term_private_company_interest_receivable_parent_code,
        short_term_bank_interest_receivable_parent_code,
        short_term_financial_institution_interest_receivable_parent_code,
        short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
        short_term_non_domiciled_company_interest_receivable_parent_code,
        long_term_individual_interest_receivable_parent_code,
        long_term_government_entity_interest_receivable_parent_code,
        long_term_private_company_interest_receivable_parent_code,
        long_term_bank_interest_receivable_parent_code,
        long_term_financial_institution_interest_receivable_parent_code,
        long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
        long_term_non_domiciled_company_interest_receivable_parent_code,
        short_term_individual_disbursed_receivable_parent_code,
        short_term_government_entity_disbursed_receivable_parent_code,
        short_term_private_company_disbursed_receivable_parent_code,
        short_term_bank_disbursed_receivable_parent_code,
        short_term_financial_institution_disbursed_receivable_parent_code,
        short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
        short_term_non_domiciled_company_disbursed_receivable_parent_code,
        long_term_individual_disbursed_receivable_parent_code,
        long_term_government_entity_disbursed_receivable_parent_code,
        long_term_private_company_disbursed_receivable_parent_code,
        long_term_bank_disbursed_receivable_parent_code,
        long_term_financial_institution_disbursed_receivable_parent_code,
        long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
        long_term_non_domiciled_company_disbursed_receivable_parent_code,
        overdue_individual_disbursed_receivable_parent_code,
        overdue_government_entity_disbursed_receivable_parent_code,
        overdue_private_company_disbursed_receivable_parent_code,
        overdue_bank_disbursed_receivable_parent_code,
        overdue_financial_institution_disbursed_receivable_parent_code,
        overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
        overdue_non_domiciled_company_disbursed_receivable_parent_code,
    } = serde_json::from_str(&data)?;

    let config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        chart_of_account_facility_omnibus_parent_code: facility_omnibus_parent_code.parse()?,
        chart_of_account_collateral_omnibus_parent_code: collateral_omnibus_parent_code.parse()?,
        chart_of_account_liquidation_proceeds_omnibus_parent_code:
            liquidation_proceeds_omnibus_parent_code.parse()?,
        chart_of_account_payments_made_omnibus_parent_code: payments_made_omnibus_parent_code
            .parse()?,
        chart_of_account_interest_added_to_obligations_omnibus_parent_code:
            interest_added_to_obligations_omnibus_parent_code.parse()?,
        chart_of_account_facility_parent_code: facility_parent_code.parse()?,
        chart_of_account_collateral_parent_code: collateral_parent_code.parse()?,
        chart_of_account_collateral_in_liquidation_parent_code:
            collateral_in_liquidation_parent_code.parse()?,
        chart_of_account_interest_income_parent_code: interest_income_parent_code.parse()?,
        chart_of_account_fee_income_parent_code: fee_income_parent_code.parse()?,
        chart_of_account_payment_holding_parent_code: payment_holding_parent_code.parse()?,
        chart_of_account_uncovered_outstanding_parent_code: uncovered_outstanding_parent_code
            .parse()?,
        chart_of_account_short_term_individual_interest_receivable_parent_code:
            short_term_individual_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_government_entity_interest_receivable_parent_code:
            short_term_government_entity_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_private_company_interest_receivable_parent_code:
            short_term_private_company_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_bank_interest_receivable_parent_code:
            short_term_bank_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
            short_term_financial_institution_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
            short_term_non_domiciled_company_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_individual_interest_receivable_parent_code:
            long_term_individual_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_government_entity_interest_receivable_parent_code:
            long_term_government_entity_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_private_company_interest_receivable_parent_code:
            long_term_private_company_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_bank_interest_receivable_parent_code:
            long_term_bank_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
            long_term_financial_institution_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
            long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code.parse()?,
        chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
            long_term_non_domiciled_company_interest_receivable_parent_code.parse()?,
        chart_of_account_short_term_individual_disbursed_receivable_parent_code:
            short_term_individual_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
            short_term_government_entity_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
            short_term_private_company_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_bank_disbursed_receivable_parent_code:
            short_term_bank_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
            short_term_financial_institution_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?,
        chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
            short_term_non_domiciled_company_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_individual_disbursed_receivable_parent_code:
            long_term_individual_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
            long_term_government_entity_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
            long_term_private_company_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_bank_disbursed_receivable_parent_code:
            long_term_bank_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
            long_term_financial_institution_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?,
        chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
            long_term_non_domiciled_company_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_individual_disbursed_receivable_parent_code:
            overdue_individual_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
            overdue_government_entity_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
            overdue_private_company_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_bank_disbursed_receivable_parent_code:
            overdue_bank_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
            overdue_financial_institution_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
            overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code.parse()?,
        chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
            overdue_non_domiciled_company_disbursed_receivable_parent_code.parse()?,
    };

    match credit
        .chart_of_accounts_integrations()
        .set_config(&Subject::System, chart, config_values)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
