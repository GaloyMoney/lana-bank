{{ config(
    materialized = 'incremental',
    unique_key = 'account_id',
) }}

with ordered as (

    select
        journal_id,
        account_id,
        currency,
        recorded_at,
    {% if target.type == 'bigquery' %}
        values,
    {% elif target.type == 'snowflake' %}
        "VALUES",
    {% endif %}
        _sdc_batched_at,
        row_number()
            over (
                partition by account_id
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "cala_balance_history") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
    {% endif %}

)

select
    journal_id,
    account_id,
    currency,
    recorded_at,
{% if target.type == 'bigquery' %}
    values,
{% elif target.type == 'snowflake' %}
    "VALUES",
{% endif %}
    _sdc_batched_at

from ordered

where order_received_desc = 1
