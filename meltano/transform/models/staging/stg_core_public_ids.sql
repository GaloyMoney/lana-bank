{{ config(
    materialized = 'incremental',
    unique_key = ['target_id'],
) }}

with ordered as (

    select
        id,
        target_id,
        created_at,
        row_number()
            over (
                partition by target_id
                order by _sdc_batched_at desc
            )
            as order_received_desc,
        _sdc_batched_at,

    from {{ source("lana", "public_core_public_ids_view") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
    {% endif %}

)

select
    * except (order_received_desc)

from ordered

where order_received_desc = 1