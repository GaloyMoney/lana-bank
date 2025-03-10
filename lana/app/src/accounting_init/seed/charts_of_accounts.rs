use crate::{
    accounting_init::{constants::*, *},
    credit_facility::{CreditFacilityAccountFactories, CreditFacilityOmnibusAccountIds},
    primitives::LedgerAccountSetId,
};

use chart_of_accounts::{
    ControlAccountCreationDetails, ControlAccountDetails, ControlSubAccountDetails,
};

pub(crate) async fn init(
    balance_sheets: &BalanceSheets,
    trial_balances: &TrialBalances,
    pl_statements: &ProfitAndLossStatements,
    cash_flow_statements: &CashFlowStatements,
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartsInit, AccountingInitError> {
    let chart_ids = &create_charts_of_accounts(chart_of_accounts).await?;

    let credit_facilities = create_credit_facilities_account_paths(
        balance_sheets,
        trial_balances,
        pl_statements,
        cash_flow_statements,
        chart_of_accounts,
        chart_ids,
    )
    .await?;

    Ok(ChartsInit {
        chart_ids: *chart_ids,
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

async fn find_or_create_control_sub_account(
    chart_of_accounts: &ChartOfAccounts,
    chart_id: ChartId,
    control_account: ControlAccountCreationDetails,
    sub_name: String,
    sub_reference: String,
) -> Result<(ControlAccountDetails, ControlSubAccountDetails), AccountingInitError> {
    let control_account = match chart_of_accounts
        .find_control_account_by_reference(chart_id, control_account.reference.to_string())
        .await?
    {
        Some(details) => details,
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
                    chart_id,
                    control_account.clone(),
                    sub_name,
                    sub_reference,
                )
                .await?
        }
    };

    Ok((control_account, control_sub_account))
}
async fn create_sub_account_as_account(
    chart_of_accounts: &ChartOfAccounts,
    details: ControlSubAccountDetails,
) -> Result<LedgerAccountId, AccountingInitError> {
    let reference = &details.reference.to_string();
    let name = &details.name.to_string();
    let description = &details.name.to_string();

    Ok(chart_of_accounts
        .transaction_account_factory(details)
        .find_or_create_transaction_account(reference, name, description)
        .await?)
}

async fn create_credit_facilities_account_paths(
    balance_sheets: &BalanceSheets,
    trial_balances: &TrialBalances,
    pl_statements: &ProfitAndLossStatements,
    cash_flow_statements: &CashFlowStatements,
    chart_of_accounts: &ChartOfAccounts,
    chart_ids: &ChartIds,
) -> Result<CreditFacilitiesSeed, AccountingInitError> {
    let (collateral_control, collateral) = find_or_create_control_sub_account(
        chart_of_accounts,
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
    trial_balances
        .add_to_trial_balance(
            OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            collateral_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_liabilities(
            OBS_BALANCE_SHEET_NAME.to_string(),
            collateral_control.account_set_id,
        )
        .await?;

    let (collateral_omnibus_control, collateral_omnibus) = find_or_create_control_sub_account(
        chart_of_accounts,
        chart_ids.off_balance_sheet,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Assets,
            name: CREDIT_FACILITIES_BANK_COLLATERAL_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_BANK_COLLATERAL_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_BANK_COLLATERAL_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_BANK_COLLATERAL_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;
    trial_balances
        .add_to_trial_balance(
            OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            collateral_omnibus_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_assets(
            OBS_BALANCE_SHEET_NAME.to_string(),
            collateral_omnibus_control.account_set_id,
        )
        .await?;
    let collateral_omnibus_account_id =
        create_sub_account_as_account(chart_of_accounts, collateral_omnibus).await?;

    let (facility_control, facility) = find_or_create_control_sub_account(
        chart_of_accounts,
        chart_ids.off_balance_sheet,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Liabilities,
            name: CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_FACILITY_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_FACILITY_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;
    trial_balances
        .add_to_trial_balance(
            OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            facility_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_liabilities(
            OBS_BALANCE_SHEET_NAME.to_string(),
            facility_control.account_set_id,
        )
        .await?;

    let (facility_omnibus_control, facility_omnibus) = find_or_create_control_sub_account(
        chart_of_accounts,
        chart_ids.off_balance_sheet,
        ControlAccountCreationDetails {
            account_set_id: LedgerAccountSetId::new(),
            category: chart_of_accounts::ChartCategory::Assets,
            name: CREDIT_FACILITIES_OMNIBUS_FACILITY_CONTROL_ACCOUNT_NAME.to_string(),
            reference: CREDIT_FACILITIES_OMNIBUS_FACILITY_CONTROL_ACCOUNT_REF.to_string(),
        },
        CREDIT_FACILITIES_OMNIBUS_FACILITY_CONTROL_SUB_ACCOUNT_NAME.to_string(),
        CREDIT_FACILITIES_OMNIBUS_FACILITY_CONTROL_SUB_ACCOUNT_REF.to_string(),
    )
    .await?;
    trial_balances
        .add_to_trial_balance(
            OBS_TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            facility_omnibus_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_assets(
            OBS_BALANCE_SHEET_NAME.to_string(),
            facility_omnibus_control.account_set_id,
        )
        .await?;
    let facility_omnibus_account_id =
        create_sub_account_as_account(chart_of_accounts, facility_omnibus).await?;

    let (disbursed_receivable_control, disbursed_receivable) = find_or_create_control_sub_account(
        chart_of_accounts,
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
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            disbursed_receivable_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_assets(
            BALANCE_SHEET_NAME.to_string(),
            disbursed_receivable_control.account_set_id,
        )
        .await?;

    let (interest_receivable_control, interest_receivable) = find_or_create_control_sub_account(
        chart_of_accounts,
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
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            interest_receivable_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_assets(
            BALANCE_SHEET_NAME.to_string(),
            interest_receivable_control.account_set_id,
        )
        .await?;
    cash_flow_statements
        .add_to_from_operations(
            CASH_FLOW_STATEMENT_NAME.to_string(),
            interest_receivable_control.account_set_id,
        )
        .await?;

    let (interest_income_control, interest_income) = find_or_create_control_sub_account(
        chart_of_accounts,
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
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            interest_income_control.account_set_id,
        )
        .await?;
    pl_statements
        .add_to_revenue(
            PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
            interest_income_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_revenue(
            BALANCE_SHEET_NAME.to_string(),
            interest_income_control.account_set_id,
        )
        .await?;
    cash_flow_statements
        .add_to_revenue(
            CASH_FLOW_STATEMENT_NAME.to_string(),
            interest_income_control.account_set_id,
        )
        .await?;

    let (fee_income_control, fee_income) = find_or_create_control_sub_account(
        chart_of_accounts,
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
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            fee_income_control.account_set_id,
        )
        .await?;
    pl_statements
        .add_to_revenue(
            PROFIT_AND_LOSS_STATEMENT_NAME.to_string(),
            fee_income_control.account_set_id,
        )
        .await?;
    balance_sheets
        .add_to_revenue(
            BALANCE_SHEET_NAME.to_string(),
            fee_income_control.account_set_id,
        )
        .await?;
    cash_flow_statements
        .add_to_revenue(
            CASH_FLOW_STATEMENT_NAME.to_string(),
            fee_income_control.account_set_id,
        )
        .await?;

    let (fee_income_adjustment_omnibus_control, fee_income_adjustment_omnibus) =
        find_or_create_control_sub_account(
            chart_of_accounts,
            chart_ids.primary,
            ControlAccountCreationDetails {
                account_set_id: LedgerAccountSetId::new(),
                category: chart_of_accounts::ChartCategory::Expenses,
                name: CREDIT_FACILITIES_OPERATIONS_NON_CASH_ADJUSTMENTS_CONTROL_ACCOUNT_NAME
                    .to_string(),
                reference: CREDIT_FACILITIES_OPERATIONS_NON_CASH_ADJUSTMENTS_CONTROL_ACCOUNT_REF
                    .to_string(),
            },
            CREDIT_FACILITIES_FEE_INCOME_ADJUSTMENT_CONTROL_SUB_ACCOUNT_NAME.to_string(),
            CREDIT_FACILITIES_FEE_INCOME_ADJUSTMENT_CONTROL_SUB_ACCOUNT_REF.to_string(),
        )
        .await?;
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            fee_income_adjustment_omnibus_control.account_set_id,
        )
        .await?;
    cash_flow_statements
        .add_to_fee_income_adjustments(
            CASH_FLOW_STATEMENT_NAME.to_string(),
            fee_income_adjustment_omnibus_control.account_set_id,
        )
        .await?;
    let fee_income_adjustment_omnibus_account_id =
        create_sub_account_as_account(chart_of_accounts, fee_income_adjustment_omnibus).await?;

    let (deposit_adjustment_omnibus_control, deposit_adjustment_omnibus) =
        find_or_create_control_sub_account(
            chart_of_accounts,
            chart_ids.primary,
            ControlAccountCreationDetails {
                account_set_id: LedgerAccountSetId::new(),
                category: chart_of_accounts::ChartCategory::Expenses,
                name: CREDIT_FACILITIES_FINANCING_NON_CASH_ADJUSTMENTS_CONTROL_ACCOUNT_NAME
                    .to_string(),
                reference: CREDIT_FACILITIES_FINANCING_NON_CASH_ADJUSTMENTS_CONTROL_ACCOUNT_REF
                    .to_string(),
            },
            CREDIT_FACILITIES_DEPOSIT_ADJUSTMENT_CONTROL_SUB_ACCOUNT_NAME.to_string(),
            CREDIT_FACILITIES_DEPOSIT_ADJUSTMENT_CONTROL_SUB_ACCOUNT_REF.to_string(),
        )
        .await?;
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            deposit_adjustment_omnibus_control.account_set_id,
        )
        .await?;
    cash_flow_statements
        .add_to_deposit_adjustments(
            CASH_FLOW_STATEMENT_NAME.to_string(),
            deposit_adjustment_omnibus_control.account_set_id,
        )
        .await?;
    let deposit_adjustment_omnibus_account_id =
        create_sub_account_as_account(chart_of_accounts, deposit_adjustment_omnibus).await?;

    let (non_cash_offset_omnibus_control, non_cash_offset_omnibus) =
        find_or_create_control_sub_account(
            chart_of_accounts,
            chart_ids.primary,
            ControlAccountCreationDetails {
                account_set_id: LedgerAccountSetId::new(),
                category: chart_of_accounts::ChartCategory::Equity,
                name: CREDIT_FACILITIES_NON_CASH_OFFSET_CONTROL_ACCOUNT_NAME.to_string(),
                reference: CREDIT_FACILITIES_NON_CASH_OFFSET_CONTROL_ACCOUNT_REF.to_string(),
            },
            CREDIT_FACILITIES_NON_CASH_OFFSET_CONTROL_SUB_ACCOUNT_NAME.to_string(),
            CREDIT_FACILITIES_NON_CASH_OFFSET_CONTROL_SUB_ACCOUNT_REF.to_string(),
        )
        .await?;
    trial_balances
        .add_to_trial_balance(
            TRIAL_BALANCE_STATEMENT_NAME.to_string(),
            non_cash_offset_omnibus_control.account_set_id,
        )
        .await?;
    let non_cash_offset_omnibus_account_id =
        create_sub_account_as_account(chart_of_accounts, non_cash_offset_omnibus).await?;

    Ok(CreditFacilitiesSeed {
        factories: CreditFacilityAccountFactories {
            collateral: chart_of_accounts.transaction_account_factory(collateral),
            facility: chart_of_accounts.transaction_account_factory(facility),
            disbursed_receivable: chart_of_accounts
                .transaction_account_factory(disbursed_receivable),
            interest_receivable: chart_of_accounts.transaction_account_factory(interest_receivable),
            interest_income: chart_of_accounts.transaction_account_factory(interest_income),
            fee_income: chart_of_accounts.transaction_account_factory(fee_income),
        },
        omnibus_ids: CreditFacilityOmnibusAccountIds {
            bank_collateral: collateral_omnibus_account_id,
            facility: facility_omnibus_account_id,
            fee_income_adjustment: fee_income_adjustment_omnibus_account_id,
            debit_account_adjustment: deposit_adjustment_omnibus_account_id,
            non_cash_offset: non_cash_offset_omnibus_account_id,
        },
    })
}
