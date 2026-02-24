{{
    config(
        unique_key=["journal_id", "account_id", "currency", "effective", "version"],
    )
}}

with
    raw as (select * from {{ source("lana", "cala_cumulative_effective_balances") }}),

    ordered as (
        select
            journal_id,
            account_id,
            currency,
            effective,
            version,
            all_time_version,
            latest_entry_id,
        values
            ,
            updated_at,
            created_at,
            timestamp_micros(
                cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
            ) as loaded_to_dw_at,
            row_number() over (
                partition by account_id, effective order by _dlt_load_id desc
            ) as order_received_desc

        from raw
    )

select * except (order_received_desc)

from ordered

where order_received_desc = 1
