{{ config(
    unique_key = ['journal_id', 'account_id', 'currency', 'version'],
) }}

with raw_stg_cala_balance_history as (select * from {{ source("lana", "cala_balance_history")}} ),

ordered as (

    select
        journal_id,
        account_id,
        currency,
        version,
        recorded_at,
        values,
        TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at,
        row_number()
            over (
                partition by account_id
                order by _dlt_load_id desc
            )
            as order_received_desc

    from raw_stg_cala_balance_history

)

select * except (order_received_desc)

from ordered

where order_received_desc = 1
