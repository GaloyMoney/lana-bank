select
    s.id as credit_facility_id,
    s.*

from {{ source("lana", "public_core_credit_facility_events_rollup_view") }} as s
