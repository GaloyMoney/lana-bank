{{ config(materialized='table') }}

with value_approved_cf as (
    select (sum(facility) / 100.0) as amount_in_usd
    from {{ ref("int_credit_facilities") }}
    where
        approval_process_concluded_approved
        and completed_recorded_at is null
),

disbursed as (
    select (sum(total_disbursed_amount) / 100.0) as amount_in_usd
    from {{ ref("int_cf_flatten") }}
    where
        disbursal_concluded_event_recorded_at_date_key != 19000101
        and completed_recorded_at is null
),

breakeven as (
    select
        cfe.event_id,
        5.53 as bench_mark,              	      -- TODO get from proper source
        cfe.terms_annual_rate,
        facility as credit_facility_limit_in_cents,
        coalesce(total_disbursed_amount, 0) as disbursal_amount_in_cents
    from {{ ref("int_cf_flatten") }} as cfe
    where
        approval_process_concluded_approved
        and completed_recorded_at is null
        and facility > 0
),

breakeven_by_cf as (
    select
        event_id,
        bench_mark,
        terms_annual_rate,
        credit_facility_limit_in_cents,
        sum(disbursal_amount_in_cents) as disbursal_amount_in_cents
    from breakeven
    group by
        event_id,
        bench_mark,
        terms_annual_rate,
        credit_facility_limit_in_cents
),

breakeven_ratio as (
    select
        event_id,
        bench_mark,
        terms_annual_rate,
        disbursal_amount_in_cents,
        credit_facility_limit_in_cents,
        bench_mark / 100.0 as bench_mark_interest_rate,
        (credit_facility_limit_in_cents / nullif(sum(credit_facility_limit_in_cents) over (), 0)) as facility_limit_ratio,
        (disbursal_amount_in_cents / nullif(credit_facility_limit_in_cents, 0)) as disbursal_ratio,
        (bench_mark / nullif(terms_annual_rate, 0)) as breakeven_disbursal_ratio
    from breakeven_by_cf
),

breakeven_prop as (
    select
        event_id,
        bench_mark,
        terms_annual_rate,
        disbursal_amount_in_cents,
        credit_facility_limit_in_cents,
        bench_mark_interest_rate,
        facility_limit_ratio,
        disbursal_ratio,
        breakeven_disbursal_ratio,
        (breakeven_disbursal_ratio * facility_limit_ratio) as prop_breakeven_disbursal_ratio,
        (disbursal_ratio * facility_limit_ratio) as prop_disbursal_ratio
    from breakeven_ratio
),

breakeven_sum as (
    select
        bench_mark,
        sum(prop_breakeven_disbursal_ratio) as breakeven_disbursal_ratio,
        sum(prop_disbursal_ratio) as disbursal_ratio
    from breakeven_prop
    group by bench_mark
)


select
    1 as order_by,
    'Value Approved CF (USD)' as kpi_title,
    'value_approved_cf_usd' as kpi_name,
    cast(amount_in_usd as {{ dbt.type_numeric() }}) as kpi_value
from value_approved_cf
union all
select
    2 as order_by,
    'Value Disbursed from Approved CF (USD)' as kpi_title,
    'value_disbursed_from_approved_cf_usd' as kpi_name,
    cast(amount_in_usd as {{ dbt.type_numeric() }}) as kpi_value
from disbursed
union all
select
    3 as order_by,
    'Value NOT-YET Dsbd from Appd CF (USD)' as kpi_title,
    'value_not_yet_dsbd_from_appd_cf_usd' as kpi_name,
    cast((v.amount_in_usd - d.amount_in_usd) as {{ dbt.type_numeric() }}) as kpi_value
from value_approved_cf as v, disbursed as d
union all
select
    4 as order_by,
    'Disbursed-to-Approved Ratio (%)' as kpi_title,
    'disbursed_to_approved_ratio_percent' as kpi_name,
    cast(((d.amount_in_usd / nullif(v.amount_in_usd, 0)) * 100.0) as {{ dbt.type_numeric() }})
        as kpi_value
from value_approved_cf as v, disbursed as d
union all
select
    5 as order_by,
    'Disbursal Ratio (%) - proportional' as kpi_title,
    'disbursal_ratio_percent_proportional' as kpi_name,
    cast((disbursal_ratio * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from breakeven_sum
union all
select
    6 as order_by,
    'Breakeven Ratio (%) - prop @' || bench_mark || '% bmk' as kpi_title,
    'breakeven_ratio_percent_prop_bmk' as kpi_name,
    cast((breakeven_disbursal_ratio * 100.0) as {{ dbt.type_numeric() }}) as kpi_value
from breakeven_sum

order by order_by
