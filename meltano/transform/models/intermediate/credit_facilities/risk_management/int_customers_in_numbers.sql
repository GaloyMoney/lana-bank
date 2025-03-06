{{ config(materialized='table') }}

with total_customers as (
    select count(distinct customer_id) as kpi_value
    from {{ ref("int_credit_facilities") }}
),

total_active_customers as (
    select count(distinct customer_id) as kpi_value
    from {{ ref("int_credit_facilities") }}
    where completed_recorded_at is null
),

approved_cf as (
    select count(distinct customer_id) as kpi_value
    from {{ ref("int_credit_facilities") }}
    where
        approval_process_concluded_approved
        and completed_recorded_at is null
),

disbursed_cf as (
    select count(distinct customer_id) as kpi_value
    from {{ ref("int_cf_flatten") }}
    where
        disbursal_concluded_event_recorded_at_date_key != 19000101
        and completed_recorded_at is null
)


select
    1 as order_by,
    'Number Customers' as kpi_title,
    'number_customers' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from total_customers
union all
select
    2 as order_by,
    'Number Active Customers' as kpi_title,
    'number_active_customers' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from total_active_customers
union all
select
    3 as order_by,
    'Number Customers w Approved CF' as kpi_title,
    'number_customers_w_approved_cf' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from approved_cf
union all
select
    4 as order_by,
    'Number Customers w Dsbd Approved CF' as kpi_title,
    'number_customers_w_dsbd_approved_cf' as kpi_name,
    cast(kpi_value as {{ dbt.type_numeric() }}) as kpi_value
from disbursed_cf

order by order_by
