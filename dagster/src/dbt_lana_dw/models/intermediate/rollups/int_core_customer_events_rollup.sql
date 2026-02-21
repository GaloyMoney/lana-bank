with
    latest_sequence as (
        select customer_id, max({{ ident('version') }}) as {{ ident('version') }}
        from {{ ref("int_core_customer_events_rollup_sequence") }}
        group by customer_id
    ),

    all_event_sequence as (
        select * from {{ ref("int_core_customer_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join latest_sequence using (customer_id, {{ ident('version') }})

    )

select *
from final
