{{ config(materialized='table') }}

select
    customer_id,
    country,
    ip_country

from {{ ref('int_customers') }}
