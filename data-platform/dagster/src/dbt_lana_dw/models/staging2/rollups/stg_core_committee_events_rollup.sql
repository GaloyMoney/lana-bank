select
    s.id as committee_id,
    s.*

from {{ source("lana", "public_core_committee_events_rollup_view") }} as s
