{{ config(
    materialized = 'incremental',
    unique_key = 'account_id',
    full_refresh = true,
) }}
-- TODO: remove full_refresh config after rollout

with ordered as (

    select
        journal_id,
        account_id,
        currency,
        recorded_at,
        values,
        _sdc_batched_at,
        row_number()
            over (
                partition by account_id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "public_cala_balance_history_view") }}

    {% if is_incremental() %}
    where _sdc_batched_at >= (select coalesce(max(_sdc_batched_at),'1900-01-01') from {{ this }} )
    {% endif %}

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
