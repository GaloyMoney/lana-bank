use chart_of_accounts::{ChartCategory, ControlSubAccountDetails};

use crate::primitives::LedgerAccountSetId;

use super::{constants::*, *};

pub(super) async fn journal(cala: &CalaLedger) -> Result<JournalInit, AccountingInitError> {
    use cala_ledger::journal::*;

    let new_journal = NewJournal::builder()
        .id(JournalId::new())
        .name("General Ledger")
        .description("General ledger for Lana")
        .code(LANA_JOURNAL_CODE)
        .build()
        .expect("new journal");

    match cala.journals().create(new_journal).await {
        Err(cala_ledger::journal::error::JournalError::CodeAlreadyExists) => {
            let journal = cala
                .journals()
                .find_by_code(LANA_JOURNAL_CODE.to_string())
                .await?;
            Ok(JournalInit {
                journal_id: journal.id,
            })
        }
        Err(e) => Err(e.into()),
        Ok(journal) => Ok(JournalInit {
            journal_id: journal.id,
        }),
    }
}

pub(super) async fn charts_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartsInit, AccountingInitError> {
    let chart_ids = &create_charts_of_accounts(chart_of_accounts).await?;

    let deposits = create_deposits_account_paths(chart_of_accounts, chart_ids).await?;

    let credit_facilities =
        create_credit_facilities_account_paths(chart_of_accounts, chart_ids).await?;

    Ok(ChartsInit {
        chart_ids: *chart_ids,
        deposits,
        credit_facilities,
    })
}

async fn create_charts_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartIds, AccountingInitError> {
    let primary = match chart_of_accounts
        .find_by_reference(CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(
                    ChartId::new(),
                    CHART_NAME.to_string(),
                    CHART_REF.to_string(),
                )
                .await?
        }
    };

    let off_balance_sheet = match chart_of_accounts
        .find_by_reference(OBS_CHART_REF.to_string())
        .await?
    {
        Some(chart) => chart,
        None => {
            chart_of_accounts
                .create_chart(
                    ChartId::new(),
                    OBS_CHART_NAME.to_string(),
                    OBS_CHART_REF.to_string(),
                )
                .await?
        }
    };
    Ok(ChartIds {
        primary: primary.id,
        off_balance_sheet: off_balance_sheet.id,
    })
}

#[allow(clippy::too_many_arguments)]
async fn create_control_sub_account(
    chart_of_accounts: &ChartOfAccounts,
    id: LedgerAccountSetId,
    chart_id: ChartId,
    category: ChartCategory,
    control_name: String,
    control_reference: String,
    sub_name: String,
    sub_reference: String,
) -> Result<ControlSubAccountDetails, AccountingInitError> {
    let control_path = match chart_of_accounts
        .find_control_account_by_reference(chart_id, control_reference.clone())
        .await?
    {
        Some(path) => path,
        None => {
            chart_of_accounts
                .create_control_account(chart_id, category, control_name, control_reference)
                .await?
        }
    };

    let control_sub_account = match chart_of_accounts
        .find_control_sub_account_by_reference(chart_id, sub_reference.clone())
        .await?
    {
        Some(account_details) => account_details,
        None => {
            chart_of_accounts
                .create_control_sub_account(id, chart_id, control_path, sub_name, sub_reference)
                .await?
        }
    };

    Ok(control_sub_account)
}

async fn create_deposits_account_paths(
    chart_of_accounts: &ChartOfAccounts,
    chart_ids: &ChartIds,
) -> Result<DepositsAccountPaths, AccountingInitError> {
    let deposits = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        chart_of_accounts::ChartCategory::Liabilities,
        DEPOSITS_CONTROL_ACCOUNT_NAME.to_string(),
        DEPOSITS_CONTROL_ACCOUNT_REF.to_string(),
        DEPOSITS_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        DEPOSITS_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    Ok(DepositsAccountPaths { deposits })
}

async fn create_credit_facilities_account_paths(
    chart_of_accounts: &ChartOfAccounts,
    chart_ids: &ChartIds,
) -> Result<CreditFacilitiesAccountPaths, AccountingInitError> {
    let collateral = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.off_balance_sheet,
        chart_of_accounts::ChartCategory::Liabilities,
        CREDIT_FACILITIES_COLLATERAL_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_COLLATERAL_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_COLLATERAL_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_COLLATERAL_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let facility = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.off_balance_sheet,
        chart_of_accounts::ChartCategory::Assets,
        CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let disbursed_receivable = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        chart_of_accounts::ChartCategory::Assets,
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let interest_receivable = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        chart_of_accounts::ChartCategory::Assets,
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let interest_income = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        chart_of_accounts::ChartCategory::Revenues,
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let fee_income = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        chart_of_accounts::ChartCategory::Revenues,
        CREDIT_FACILITIES_FEE_INCOME_CONTROL_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FEE_INCOME_CONTROL_ACCOUNT_REF.to_string(),
        CREDIT_FACILITIES_FEE_INCOME_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FEE_INCOME_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    Ok(CreditFacilitiesAccountPaths {
        collateral,
        facility,
        disbursed_receivable,
        interest_receivable,
        interest_income,
        fee_income,
    })
}
