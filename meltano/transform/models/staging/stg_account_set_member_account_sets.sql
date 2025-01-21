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

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
