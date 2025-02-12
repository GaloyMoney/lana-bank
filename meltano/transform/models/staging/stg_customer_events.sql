{{ config(
    materialized = 'incremental',
    unique_key = ['id', 'sequence'],
    full_refresh = true,
) }}
-- TODO: remove full_refresh config after rollout

with ordered as (

    select
        id,
        sequence,
        event_type,
        event,
        recorded_at,
        row_number()
            over (
                partition by id, sequence
                order by _sdc_received_at desc
            )
            as order_received_desc

    from {{ source("lana", "public_customer_events_view") }}

    {% if is_incremental() %}
    where recorded_at >= (select coalesce(max(recorded_at),'1900-01-01') from {{ this }} )
    {% endif %}

)

select
    * except (order_received_desc),
    safe.parse_json(event) as parsed_event

from ordered

where order_received_desc = 1
