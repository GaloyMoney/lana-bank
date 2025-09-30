{{ config(
    materialized = 'incremental',
    unique_key = ['id'],
) }}

with ordered as (

    select
        id,
        code,
        name,
        normal_balance_type,
        -- TODO: need fixing: where did old latest_values go?
        -- It held "$.config.is_account_set" flag
        null as latest_values,
        created_at,
        _sdc_batched_at,
        row_number()
            over (
                partition by id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "public_cala_accounts_view") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (
                select coalesce(max(_sdc_batched_at), '1900-01-01')
                from {{ this }}
            )
    {% endif %}

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1

qualify _sdc_batched_at = max(_sdc_batched_at) over ()
