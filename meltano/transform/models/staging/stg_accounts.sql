{{ config(
    materialized = 'incremental',
    unique_key = 'id',
) }}

with ordered as (

    select
        id,
        code,
        name,
        normal_balance_type,
        latest_values,
        created_at,
        _sdc_batched_at,
        row_number()
            over (
                partition by id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "cala_accounts") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
    {% endif %}

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
