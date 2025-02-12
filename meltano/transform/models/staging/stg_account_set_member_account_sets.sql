{{ config(
    materialized = 'incremental',
    unique_key = 'account_set_id',
    full_refresh = true,
) }}
-- TODO: remove full_refresh config after rollout

with ordered as (

    select
        account_set_id,
        member_account_set_id,
        created_at,
        row_number()
            over (
                partition by account_set_id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from
        {{ source("lana", "public_cala_account_set_member_account_sets_view") }}

    {% if is_incremental() %}
    where created_at >= (select coalesce(max(created_at),'1900-01-01') from {{ this }} )
    {% endif %}

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
