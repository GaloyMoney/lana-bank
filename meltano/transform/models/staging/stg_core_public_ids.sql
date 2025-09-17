{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'target_id'],
) }}

with ordered as (

    select
        id,
        target_id,
        created_at,
        row_number()
            over (
                partition by id, target_id
                order by _sdc_received_at desc
            )
            as order_received_desc

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
