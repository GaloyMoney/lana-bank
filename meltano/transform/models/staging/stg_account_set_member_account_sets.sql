{{ config(
    materialized = 'incremental',
    unique_key = 'account_set_id',
) }}

with ordered as (

    select
        account_set_id,
        member_account_set_id,
        created_at,
        _sdc_batched_at,
        row_number()
            over (
                partition by account_set_id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from
        {{ source("lana", "cala_account_set_member_account_sets") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
    {% endif %}

)

select
    account_set_id,
    member_account_set_id,
    created_at,
    _sdc_batched_at,

from ordered

where order_received_desc = 1
