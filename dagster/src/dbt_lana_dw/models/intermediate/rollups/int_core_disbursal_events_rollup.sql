with
    latest_sequence as (
        select disbursal_id, max({{ ident('version') }}) as {{ ident('version') }}
        from {{ ref("int_core_disbursal_events_rollup_sequence") }}
        group by disbursal_id
    ),

    all_event_sequence as (
        select * from {{ ref("int_core_disbursal_events_rollup_sequence") }}
    ),

    final as (
        select *
        from all_event_sequence
        inner join latest_sequence using (disbursal_id, {{ ident('version') }})

    )

select *
from final
