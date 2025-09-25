import dagster as dg

from lana_pipelines.assets import build_lana_source_asset, build_lana_to_dw_el_asset, build_dbt_assets, build_generate_es_report_asset
from lana_pipelines.resources import dbt_resource


def build_definitions():

    lana_to_dw_el_tables = (
        # "cala_balance_history",
        # "cala_accounts",
        # "cala_account_sets",
        # "cala_account_set_member_accounts",
        # "cala_account_set_member_account_sets",
        # "core_chart_events",
        # "core_collateral_events",
        # "core_credit_facility_events",
        # "core_credit_facility_repayment_plans",
        # "core_deposit_accounts",
        # "core_deposit_events",
        # "core_disbursal_events",
        # "core_interest_accrual_cycle_events",
        # "core_obligation_events",
        # "core_obligation_installment_events",
        # "core_payment_events",
        # "core_withdrawal_events",
        # "core_withdrawals",
        # "core_customer_events",
        # "core_user_events_rollup",
        # "core_role_events_rollup",
        # "core_permission_set_events_rollup",
        # "core_approval_process_events_rollup",
        # "core_committee_events_rollup",
        # "core_policy_events_rollup",
        # "core_customer_events_rollup",
        "core_deposit_account_events_rollup",
        "core_deposit_events_rollup",
        "core_withdrawal_events_rollup",
        "core_public_ids",
        # "core_custodian_events_rollup",
        # "core_collateral_events_rollup",
        # "core_credit_facility_events_rollup",
        # "core_disbursal_events_rollup",
        # "core_interest_accrual_cycle_events_rollup",
        # "core_obligation_events_rollup",
        # "core_obligation_installment_events_rollup",
        # "core_payment_events_rollup",
        # "core_terms_template_events_rollup",
        # "core_chart_events_rollup",
        # "core_manual_transaction_events_rollup",
        # "core_document_events_rollup",
        # "core_liquidation_process_events_rollup",
    )

    lana_source_assets = [
        build_lana_source_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    lana_to_dw_el_assets = [
        build_lana_to_dw_el_asset(table_name=table_name)
        for table_name in lana_to_dw_el_tables
    ]

    dbt_assets = [build_dbt_assets()]

    generate_es_report_asset = [build_generate_es_report_asset()]

    lana_to_dw_el_job = dg.define_asset_job(
        name="lana_to_dw_el_job",
        selection=lana_to_dw_el_assets,
        # Below is a silly hardcode to prevent running into rate limiting with
        # BQ: we should research how to avoid it because it gets triggered 
        # with rather low volumes.
        config={
            "execution": {
                "config": {
                    "multiprocess": {"max_concurrent": 4}, 
                }
            }
        },
    )

    build_dbt_job = dg.define_asset_job(
        name="build_dbt_job",
        selection=dbt_assets,
    )

    build_generate_es_report_job = dg.define_asset_job(
        name="generate_es_report_job",
        selection=generate_es_report_asset,
    )

    all_assets = lana_source_assets + lana_to_dw_el_assets + dbt_assets + generate_es_report_asset
    all_jobs = [lana_to_dw_el_job, build_dbt_job, build_generate_es_report_job]
    all_resources = {
        "dbt": dbt_resource
    }

    return dg.Definitions(assets=all_assets, jobs=all_jobs, resources=all_resources)


defs = build_definitions()
