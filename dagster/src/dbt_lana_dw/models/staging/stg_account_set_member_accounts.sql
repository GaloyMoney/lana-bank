{{
    config(
        unique_key=["account_set_id", "member_account_id"],
    )
}}

with
    raw_stg_cala_account_set_member_accounts as (
        select * from {{ source("lana", "cala_account_set_member_accounts") }}
    ),

    ordered as (

        select
            account_set_id,
            member_account_id,
            transitive,
            created_at,
            timestamp_micros(
                cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
            ) as loaded_to_dw_at,
            row_number() over (
                partition by account_set_id, member_account_id
                order by _dlt_load_id desc
            ) as order_received_desc

        from raw_stg_cala_account_set_member_accounts
    )

select * except (order_received_desc)

from ordered

where order_received_desc = 1
