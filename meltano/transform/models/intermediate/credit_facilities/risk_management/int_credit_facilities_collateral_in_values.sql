{{ config(materialized='table') }}

with flatten_collateral as (
    select *
    from {{ ref('int_cf_flatten') }}
),

collateral_quantity as (
    select (sum(total_collateral) / 100000000.0) as total_collateral_quantity_usd
    from flatten_collateral
    where completed_recorded_at is null
),

collateral_value as (
    select
        sum(total_collateral_value_usd) as total_collateral_value_usd,
        sum(initial_collateral_value_usd) as initial_collateral_value_usd
    from flatten_collateral
    where completed_recorded_at is null
),

value_approved_cf as (
    select (sum(facility) / 100.0) as total_value_approved_in_usd
    from flatten_collateral
    where
        approval_process_concluded_approved
        and completed_recorded_at is null
),

value_disbursed as (
    select (sum(total_disbursed_amount) / 100.0) as total_value_disbursed_in_usd
    from flatten_collateral
    where
        disbursal_concluded_event_recorded_at_date_key != 19000101
        and completed_recorded_at is null
),

agg_facility_cvl as (
    select
        ((total_collateral_value_usd / nullif(total_value_approved_in_usd, 0)) * 100.0)
            as aggregated_facility_cvl,
        ((initial_collateral_value_usd / nullif(total_value_approved_in_usd, 0)) * 100.0
        ) as aggregated_initial_facility_cvl
    from collateral_value, value_approved_cf
),

agg_disbursed_cvl as (
    select
        ((total_collateral_value_usd / nullif(total_value_disbursed_in_usd, 0)) * 100.0
        ) as aggregated_disbursed_cvl
    from collateral_value, value_disbursed
),

cvl_implied_prices as (
    select
        (
            sum((average_initial_price_usd * total_collateral)) /
            nullif(sum(total_collateral), 0)
        ) as aggregated_average_initial_price_usd,

        {% if target.type == 'bigquery' %}
            array_agg(
                last_btc_price_usd
                ignore nulls
                order by recorded_at desc limit 1)[
                safe_ordinal(1)
            ] as last_btc_price_usd,
        {% elif target.type == 'snowflake' %}
            GET(array_agg(last_btc_price_usd) WITHIN GROUP(order by recorded_at desc), 0) as last_btc_price_usd,
        {% endif %}

        (
            sum((facility_margin_call_price_usd * total_collateral)) /
            nullif(sum(total_collateral), 0)
        ) as aggregated_facility_margin_call_price_usd,
        (
            sum((disbursed_margin_call_price_usd * total_collateral)) /
            nullif(sum(total_collateral), 0)
        ) as aggregated_disbursed_margin_call_price_usd,
        (
            sum((facility_liquidation_price_usd * total_collateral)) /
            nullif(sum(total_collateral), 0)
        ) as aggregated_facility_liquidation_price_usd,
        (
            sum((disbursed_liquidation_price_usd * total_collateral)) /
            nullif(sum(total_collateral), 0)
        ) as aggregated_disbursed_liquidation_price_usd
    from flatten_collateral
),

btc_price_simulation as (
    select *
    from {{ ref('int_btc_price_simulation') }}
),

sim_4s_implied_prices as (
    select
        sim.*,
        ((1.0 + sim.period_1_day_loss_percent) / px.last_btc_price_usd)
            as period_1_day_loss,
        ((1.0 + sim.period_3_day_loss_percent) / px.last_btc_price_usd)
            as period_3_day_loss,
        ((1.0 + sim.period_1_week_loss_percent) / px.last_btc_price_usd)
            as period_1_week_loss,
        ((1.0 + sim.period_2_week_loss_percent) / px.last_btc_price_usd)
            as period_2_week_loss,
        ((1.0 + sim.period_3_week_loss_percent) / px.last_btc_price_usd)
            as period_3_week_loss,
        ((1.0 + sim.period_1_month_loss_percent) / px.last_btc_price_usd)
            as period_1_month_loss
    from btc_price_simulation as sim, cvl_implied_prices as px
    where sim.sigma_level = 4
),

sim_5s_implied_prices as (
    select
        sim.*,
        ((1.0 + sim.period_1_day_loss_percent) / px.last_btc_price_usd)
            as period_1_day_loss,
        ((1.0 + sim.period_3_day_loss_percent) / px.last_btc_price_usd)
            as period_3_day_loss,
        ((1.0 + sim.period_1_week_loss_percent) / px.last_btc_price_usd)
            as period_1_week_loss
    from btc_price_simulation as sim, cvl_implied_prices as px
    where sim.sigma_level = 5
),

sim_6s_implied_prices as (
    select
        sim.*,
        ((1.0 + sim.period_1_day_loss_percent) / px.last_btc_price_usd)
            as period_1_day_loss,
        ((1.0 + sim.period_3_day_loss_percent) / px.last_btc_price_usd)
            as period_3_day_loss,
        ((1.0 + sim.period_1_week_loss_percent) / px.last_btc_price_usd)
            as period_1_week_loss
    from btc_price_simulation as sim, cvl_implied_prices as px
    where sim.sigma_level = 6
),

agg_liquidation_cash_flows_tvm_risk as (
    select *
    from {{ ref('int_cf_agg_liquidation_cash_flows_tvm_risk') }}
)


select
    10 as order_by,
    'Collateral Quantity (BTC)' as kpi_title,
    'collateral_quantity_btc' as kpi_name,
    cast(total_collateral_quantity_usd as {{ dbt.type_numeric() }}) as kpi_value
from collateral_quantity
union all
select
    20 as order_by,
    'Collateral Initial Value (USD)' as kpi_title,
    'collateral_initial_value_usd' as kpi_name,
    cast(initial_collateral_value_usd as {{ dbt.type_numeric() }}) as kpi_value
from collateral_value
union all
select
    30 as order_by,
    'Collateral Current Value (USD)' as kpi_title,
    'collateral_current_value_usd' as kpi_name,
    cast(total_collateral_value_usd as {{ dbt.type_numeric() }}) as kpi_value
from collateral_value
union all
select
    40 as order_by,
    'Value Approved CF (USD)' as kpi_title,
    'value_approved_cf_usd' as kpi_name,
    cast(total_value_approved_in_usd as {{ dbt.type_numeric() }}) as kpi_value
from value_approved_cf
union all
select
    50 as order_by,
    'Agg Initial Facility CVL (%)' as kpi_title,
    'agg_initial_facility_cvl_percent' as kpi_name,
    cast(aggregated_initial_facility_cvl as {{ dbt.type_numeric() }}) as kpi_value
from agg_facility_cvl
union all
select
    60 as order_by,
    'Agg Current Facility CVL (%)' as kpi_title,
    'agg_current_facility_cvl_percent' as kpi_name,
    cast(aggregated_facility_cvl as {{ dbt.type_numeric() }}) as kpi_value
from agg_facility_cvl
union all
select
    70 as order_by,
    'Value Disbursed from Approved CF (USD)' as kpi_title,
    'value_disbursed_from_approved_cf_usd' as kpi_name,
    cast(total_value_disbursed_in_usd as {{ dbt.type_numeric() }}) as kpi_value
from value_disbursed
union all
select
    80 as order_by,
    'Agg Disbursed CVL (%)' as kpi_title,
    'agg_disbursed_cvl_percent' as kpi_name,
    cast(aggregated_disbursed_cvl as {{ dbt.type_numeric() }}) as kpi_value
from agg_disbursed_cvl
union all
select
    85 as order_by,
    'Agg Average Initial Price (USD)' as kpi_title,
    'agg_average_initial_price_usd' as kpi_name,
    cast(aggregated_average_initial_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    86 as order_by,
    'Current Price (USD)' as kpi_title,
    'current_price_usd' as kpi_name,
    cast(last_btc_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    90 as order_by,
    'Agg Facility Margin Call Price (USD)' as kpi_title,
    'agg_facility_margin_call_price_usd' as kpi_name,
    cast(aggregated_facility_margin_call_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    91 as order_by,
    'Agg Facility Margin Call Price (%)' as kpi_title,
    'agg_facility_margin_call_price_percent' as kpi_name,
    cast(
        (((aggregated_facility_margin_call_price_usd / nullif(last_btc_price_usd, 0)) - 1.0) * 100.0) as {{ dbt.type_numeric() }}
    ) as kpi_value
from cvl_implied_prices
union all
select
    100 as order_by,
    'Agg Disbursed Margin Call Price (USD)' as kpi_title,
    'agg_disbursed_margin_call_price_usd' as kpi_name,
    cast(aggregated_disbursed_margin_call_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    101 as order_by,
    'Agg Disbursed Margin Call Price (%)' as kpi_title,
    'agg_disbursed_margin_call_price_percent' as kpi_name,
    cast(
        (((aggregated_disbursed_margin_call_price_usd / nullif(last_btc_price_usd, 0)) - 1.0) * 100.0) as {{ dbt.type_numeric() }}
    ) as kpi_value
from cvl_implied_prices
union all
select
    110 as order_by,
    'Agg Facility Liquidation Price (USD)' as kpi_title,
    'agg_facility_liquidation_price_usd' as kpi_name,
    cast(aggregated_facility_liquidation_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    111 as order_by,
    'Agg Facility Liquidation Price (%)' as kpi_title,
    'agg_facility_liquidation_price_percent' as kpi_name,
    cast(
        (((aggregated_facility_liquidation_price_usd / nullif(last_btc_price_usd, 0)) - 1.0) * 100.0) as {{ dbt.type_numeric() }}
    ) as kpi_value
from cvl_implied_prices
union all
select
    120 as order_by,
    'Agg Disbursed Liquidation Price (USD)' as kpi_title,
    'agg_disbursed_liquidation_price_usd' as kpi_name,
    cast(aggregated_disbursed_liquidation_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from cvl_implied_prices
union all
select
    121 as order_by,
    'Agg Disbursed Liquidation Price (%)' as kpi_title,
    'agg_disbursed_liquidation_price_percent' as kpi_name,
    cast(
        (((aggregated_disbursed_liquidation_price_usd / nullif(last_btc_price_usd, 0)) - 1.0) * 100.0) as {{ dbt.type_numeric() }}
    ) as kpi_value
from cvl_implied_prices
union all
select
    124 as order_by,
    'Highest Single Dsbd Liq Price (USD)' as kpi_title,
    'highest_single_dsbd_liq_price_usd' as kpi_name,
    cast(max_disbursed_liquidation_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    125 as order_by,
    'Average Single Dsbd Liq Price (USD)' as kpi_title,
    'average_single_dsbd_liq_price_usd' as kpi_name,
    cast(avg_disbursed_liquidation_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    126 as order_by,
    'Lowest Single Dsbd Liq Price (USD)' as kpi_title,
    'lowest_single_dsbd_liq_price_usd' as kpi_name,
    cast(min_disbursed_liquidation_price_usd as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    127 as order_by,
    'Highest Price Liq PV Impact (USD)' as kpi_title,
    'highest_price_liq_pv_impact_usd' as kpi_name,
    cast(max_liquidation_pv_impact as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    128 as order_by,
    'Average Price Liq PV Impact (USD)' as kpi_title,
    'average_price_liq_pv_impact_usd' as kpi_name,
    cast(avg_liquidation_pv_impact as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    129 as order_by,
    'Lowest Price Liq PV Impact (USD)' as kpi_title,
    'lowest_price_liq_pv_impact_usd' as kpi_name,
    cast(min_liquidation_pv_impact as {{ dbt.type_numeric() }}) as kpi_value
from agg_liquidation_cash_flows_tvm_risk
union all
select
    130 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_usd' as kpi_name,
    cast(period_1_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    131 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_percent' as kpi_name,
    cast((period_1_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    132 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_usd' as kpi_name,
    cast(period_1_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    133 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_percent' as kpi_name,
    cast((period_1_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    134 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_usd' as kpi_name,
    cast(period_1_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    135 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_day_price_percent' as kpi_name,
    cast((period_1_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    140 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_usd' as kpi_name,
    cast(period_3_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    141 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_percent' as kpi_name,
    cast((period_3_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    142 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_usd' as kpi_name,
    cast(period_3_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    143 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_percent' as kpi_name,
    cast((period_3_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    144 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_usd' as kpi_name,
    cast(period_3_day_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    145 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-day Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_day_price_percent' as kpi_name,
    cast((period_3_day_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    150 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_usd' as kpi_name,
    cast(period_1_week_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    151 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_percent' as kpi_name,
    cast((period_1_week_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    152 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_usd' as kpi_name,
    cast(period_1_week_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    153 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_percent' as kpi_name,
    cast((period_1_week_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_5s_implied_prices
union all
select
    154 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_usd' as kpi_name,
    cast(period_1_week_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    155 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-week Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_week_price_percent' as kpi_name,
    cast((period_1_week_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_6s_implied_prices
union all
select
    160 as order_by,
    'Simulated ' || sigma_level || 'σ, 2-week Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_2_week_price_usd' as kpi_name,
    cast(period_2_week_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    161 as order_by,
    'Simulated ' || sigma_level || 'σ, 2-week Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_2_week_price_percent' as kpi_name,
    cast((period_2_week_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    170 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-week Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_week_price_usd' as kpi_name,
    cast(period_3_week_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    171 as order_by,
    'Simulated ' || sigma_level || 'σ, 3-week Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_3_week_price_percent' as kpi_name,
    cast((period_3_week_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    180 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-month Price (USD)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_month_price_usd' as kpi_name,
    cast(period_1_month_loss as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices
union all
select
    181 as order_by,
    'Simulated ' || sigma_level || 'σ, 1-month Price (%)' as kpi_title,
    'simulated_' || sigma_level || 'σ_1_month_price_percent' as kpi_name,
    cast((period_1_month_loss_percent * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from sim_4s_implied_prices

order by order_by
