{{ config(materialized='table') }}

with tvm_risk as (
    select *
    from {{ ref('int_cf_agg_projected_cash_flows_tvm_risk') }}
),

flatten as (
    select
        (sum(total_collateral) / 100000000.0) as total_collateral_btc,
        max(disbursed_liquidation_price_usd) as max_disbursed_liquidation_price_usd,
        avg(disbursed_liquidation_price_usd) as avg_disbursed_liquidation_price_usd,
        min(disbursed_liquidation_price_usd) as min_disbursed_liquidation_price_usd,
        (sum(total_collateral) / 100000000.0 * max(disbursed_liquidation_price_usd))
            as max_disbursed_liquidation_cashflow,
        (sum(total_collateral) / 100000000.0 * avg(disbursed_liquidation_price_usd))
            as avg_disbursed_liquidation_cashflow,
        (sum(total_collateral) / 100000000.0 * min(disbursed_liquidation_price_usd))
            as min_disbursed_liquidation_cashflow
    from {{ ref('int_cf_flatten') }}
)

select
    npv,
    pv,

    total_collateral_btc,
    max_disbursed_liquidation_price_usd,
    avg_disbursed_liquidation_price_usd,
    min_disbursed_liquidation_price_usd,

    max_disbursed_liquidation_cashflow as max_liquidation_pv,
    avg_disbursed_liquidation_cashflow as avg_liquidation_pv,
    min_disbursed_liquidation_cashflow as min_liquidation_pv,

    (max_disbursed_liquidation_cashflow - pv) as max_liquidation_pv_impact,
    (avg_disbursed_liquidation_cashflow - pv) as avg_liquidation_pv_impact,
    (min_disbursed_liquidation_cashflow - pv) as min_liquidation_pv_impact,

    (npv + (max_disbursed_liquidation_cashflow - pv)) as max_liquidation_npv,
    (npv + (avg_disbursed_liquidation_cashflow - pv)) as avg_liquidation_npv,
    (npv + (min_disbursed_liquidation_cashflow - pv)) as min_liquidation_npv
from tvm_risk, flatten
