{{ config(materialized='table') }}

with final as (
    select * from {{ ref("int_cf_agg_projected_cash_flows_tvm_risk") }}
)

select
    0 as order_by,
    'PV Disbursal Cash Flows (USD)' as kpi_title,
    'pv_disbursal_cash_flows_usd' as kpi_name,
    cast(disbursal_pv as numeric) as kpi_value
from final
union all
select
    1 as order_by,
    'PV Future Cash Flows (USD)' as kpi_title,
    'pv_future_cash_flows_usd' as kpi_name,
    cast(pv as numeric) as kpi_value
from final
union all
select
    2 as order_by,
    'Net PV Dsbd & Future Cash Flows (USD)' as kpi_title,
    'net_pv_dsbd_future_cash_flows_usd' as kpi_name,
    cast(npv as numeric) as kpi_value
from final
union all
select
    3 as order_by,
    'YTM (%)' as kpi_title,
    'ytm_percent' as kpi_name,
    cast(safe_multiply(ytm, 100.0) as numeric) as kpi_value
from final
union all
select
    4 as order_by,
    'YTM @ Disbursal PV (%)' as kpi_title,
    'ytm_disbursal_pv_percent' as kpi_name,
    cast(safe_multiply(ytm_from_price, 100.0) as numeric) as kpi_value
from final
union all
select
    5 as order_by,
    'Macaulay Duration' as kpi_title,
    'macaulay_duration_number' as kpi_name,
    cast(mac_duration as numeric) as kpi_value
from final
union all
select
    6 as order_by,
    'Macaulay Duration (Date)' as kpi_title,
    'macaulay_duration_date' as kpi_name,
    cast(format_date('%Y%m%d', mac_duration_date) as numeric) as kpi_value
from final
union all
select
    7 as order_by,
    'Dollar Value [DV01] (USD)' as kpi_title,
    'dollar_value_dv01_usd' as kpi_name,
    cast(dv01 as numeric) as kpi_value
from final
union all
select
    8 as order_by,
    'PV @ DV01 (USD)' as kpi_title,
    'pv_dv01_usd' as kpi_name,
    cast(pv_at_dv01 as numeric) as kpi_value
from final

order by order_by
