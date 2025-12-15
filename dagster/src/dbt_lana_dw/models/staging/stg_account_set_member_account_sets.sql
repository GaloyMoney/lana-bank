{{ config(
    unique_key = ['account_set_id', 'member_account_set_id'],
) }}

with raw_stg_cala_account_set_member_account_sets as (select * from {{ source("lana", " ")}} ),

ordered as (

    select
        account_set_id,
        member_account_set_id,
        created_at,
        TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at,
        row_number()
            over (
                partition by account_set_id
                order by _dlt_load_id desc
            )
            as order_received_desc

    from raw_stg_cala_account_set_member_account_sets
)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
