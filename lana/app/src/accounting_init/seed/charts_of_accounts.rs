use chart_of_accounts::{
    ControlAccountCreationDetails, ControlAccountDetails, ControlSubAccountDetails,
};

use crate::{
    accounting_init::{constants::*, *},
    primitives::LedgerAccountSetId,
};

pub(crate) async fn init(
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

async fn create_control_sub_account(
    chart_of_accounts: &ChartOfAccounts,
    id: LedgerAccountSetId,
    chart_id: ChartId,
    control_account: ControlAccountCreationDetails,
    sub_name: String,
    sub_reference: String,
) -> Result<ControlSubAccountDetails, AccountingInitError> {
    let control_account = match chart_of_accounts
        .find_control_account_by_reference(chart_id, control_account.reference.to_string())
        .await?
    {
        Some(path) => ControlAccountDetails {
            path,
            account_set_id: control_account.account_set_id,
            name: control_account.name.to_string(),
            reference: control_account.reference.to_string(),
        },
        None => {
            chart_of_accounts
                .create_control_account(
                    control_account.account_set_id,
                    chart_id,
                    control_account.category,
                    control_account.name,
                    control_account.reference,
                )
                .await?
        }
    };

    let control_sub_account = match chart_of_accounts
        .find_control_sub_account_by_reference(chart_id, sub_reference.to_string())
        .await?
    {
        Some(account_details) => account_details,
        None => {
            chart_of_accounts
                .create_control_sub_account(
                    id,
                    chart_id,
                    control_account.path,
                    sub_name,
                    sub_reference,
                )
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
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Liabilities,
            name: DEPOSITS_CONTROL_ACCOUNT_NAME.to_string(),
            reference: DEPOSITS_CONTROL_ACCOUNT_REF.to_string(),
        },
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
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Liabilities,
            name: CREDIT_FACILITIES_COLLATERAL_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_COLLATERAL_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_COLLATERAL_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_COLLATERAL_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let facility = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.off_balance_sheet,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Assets,
            name: CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let disbursed_receivable = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Assets,
            name: CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_DISBURSED_RECEIVABLE_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let interest_receivable = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Assets,
            name: CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_RECEIVABLE_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let interest_income = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Revenues,
            name: CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_INTEREST_INCOME_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;

    let fee_income = create_control_sub_account(
        chart_of_accounts,
        LedgerAccountSetId::new(),
        chart_ids.primary,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Revenues,
            name: CREDIT_FACILITIES_FEE_INCOME_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_FEE_INCOME_CONTROL_ACCOUNT_REF.to_string(),
        },
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
