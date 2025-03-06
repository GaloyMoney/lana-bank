{{ config(materialized='table') }}

with approved as (
    select count(distinct event_id) as kpi_value
    from {{ ref("int_credit_facilities") }}
    where
        approval_process_concluded_approved
        and completed_recorded_at is null
),

total as (
    select count(distinct event_id) as kpi_value
    from {{ ref("int_credit_facilities") }}
    where completed_recorded_at is null
)


select
    1 as order_by,
    'Number Approved CF' as kpi_title,
    'number_approved_cf' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from approved
union all
select
    2 as order_by,
    'Number CF' as kpi_title,
    'number_cf' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from total
union all
select
    3 as order_by,
    'Approval Rate (%)' as kpi_title,
    'approval_rate_percent' as kpi_name,
    cast(a.kpi_value / nullif(t.kpi_value, 0) * 100.0 as {{ dbt.type_numeric() }}) as kpi_value
from approved as a, total as t

order by order_by
