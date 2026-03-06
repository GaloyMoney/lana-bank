-- Each chart node must have exactly one "initialized" event.
-- Multiple initialized events per node would indicate a re-initialization,
-- which would break the daily balance pipeline's date spine lower bound.
with
    duplicates as (
        select id, count(*) as init_count
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
        group by id
        having count(*) > 1
    )

select *
from duplicates
