{{
    config(
        unique_key=["id"],
    )
}}

with
    raw_stg_cala_accounts as (select * from {{ source("lana", "cala_accounts") }}),

    ordered as (

        select
            id,
            code,
            name,
            normal_balance_type,
            created_at,
            timestamp_micros(
                cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
            ) as loaded_to_dw_at,
            row_number() over (
                partition by id order by _dlt_load_id desc
            ) as order_received_desc

        from raw_stg_cala_accounts
    )

select * except (order_received_desc)

from ordered

where order_received_desc = 1
