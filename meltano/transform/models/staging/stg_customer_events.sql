{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'sequence'],
) }}

with ordered as (

    select
        id,
        sequence,
        event_type,
        event,
        recorded_at,
        _sdc_batched_at,
        row_number()
            over (
                partition by id, sequence
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "customer_events") }}

    {% if is_incremental() %}
        where
            _sdc_batched_at >= (select coalesce(max(_sdc_batched_at), '1900-01-01') from {{ this }})
    {% endif %}

)

select
    id,
    sequence,
    event_type,
    event,
    recorded_at,
    _sdc_batched_at,
{% if target.type == 'bigquery' %}
    safe.parse_json(event) as parsed_event
{% elif target.type == 'snowflake' %}
    TRY_PARSE_JSON(event) as parsed_event
{% endif %}

from ordered

where order_received_desc = 1
