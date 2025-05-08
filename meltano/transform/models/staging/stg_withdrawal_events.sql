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

    from {{ source("lana", "public_core_withdrawal_events_view") }}

    where _sdc_batched_at >= (
        select coalesce(max(_sdc_batched_at), '1900-01-01')
        from {{ ref('stg_core_chart_events') }}
        where event_type = 'initialized'
    )

)

select
    * except (order_received_desc),
    safe.parse_json(event) as parsed_event

from ordered

where order_received_desc = 1
