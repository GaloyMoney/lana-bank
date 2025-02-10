{{ config(materialized='table') }}

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

    from {{ source("lana", "public_credit_facility_events_view") }}

)

select
    * except (order_received_desc),
    safe.parse_json(event) as parsed_event

from ordered

where order_received_desc = 1
