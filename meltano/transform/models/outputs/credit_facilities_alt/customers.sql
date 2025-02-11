{{ config(materialized='table') }}

select
    customer_id,
    country

from {{ ref('int_customers') }}
