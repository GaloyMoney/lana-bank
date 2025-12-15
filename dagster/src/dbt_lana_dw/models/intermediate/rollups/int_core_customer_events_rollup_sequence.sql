{{ config(
    unique_key = ['customer_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_customer_events_rollup') }} as s
),

transformed as (
    select
        * except (
            customer_id,
            created_at,
            modified_at
        ),
        customer_id,
        created_at as customer_created_at,

        modified_at as customer_modified_at
    from source
)

select * from transformed
