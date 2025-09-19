select
    s.id as terms_template_id,
    s.*

from {{ source("lana", "public_core_terms_template_events_rollup_view") }} as s
