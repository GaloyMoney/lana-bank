import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset


def build_definitions():

    lana_to_dw_el_tables = (
        "cala_balance_history",
        "cala_accounts",
        "cala_account_sets",
        "cala_account_set_member_accounts",
        "cala_account_set_member_account_sets",
        "core_chart_events",
        "core_collateral_events",
        "core_credit_facility_events",
        "core_credit_facility_repayment_plans",
        "core_deposit_accounts",
        "core_deposit_events",
        "core_disbursal_events",
        "core_interest_accrual_cycle_events",
        "core_obligation_events",
        "core_obligation_installment_events",
        "core_payment_events",
        "core_withdrawal_events",
        "core_withdrawals",
        "core_customer_events",
        "core_user_events_rollup",
        "core_role_events_rollup",
        "core_permission_set_events_rollup",
        "core_approval_process_events_rollup",
        "core_committee_events_rollup",
        "core_policy_events_rollup",
        "core_customer_events_rollup",
        "core_deposit_account_events_rollup",
        "core_deposit_events_rollup",
        "core_withdrawal_events_rollup",
        "core_custodian_events_rollup",
        "core_collateral_events_rollup",
        "core_credit_facility_events_rollup",
        "core_disbursal_events_rollup",
        "core_interest_accrual_cycle_events_rollup",
        "core_obligation_events_rollup",
        "core_obligation_installment_events_rollup",
        "core_payment_events_rollup",
        "core_terms_template_events_rollup",
        "core_chart_events_rollup",
        "core_manual_transaction_events_rollup",
        "core_document_events_rollup",
        "core_liquidation_process_events_rollup",
    )

    lana_source_assets = [
        build_lana_source_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_assets = [
        build_lana_to_dw_el_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_job = dg.define_asset_job(
        "lana_to_dw_el_job", selection=lana_to_dw_el_assets
    )

    all_assets = lana_source_assets + lana_to_dw_el_assets
    all_jobs = [lana_to_dw_el_job]

    return dg.Definitions(assets=all_assets, jobs=all_jobs)


defs = build_definitions()
