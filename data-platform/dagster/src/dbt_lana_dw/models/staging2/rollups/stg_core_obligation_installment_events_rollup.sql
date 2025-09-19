select
    s.id as obligation_installment_id,
    s.*

from {{ source("lana", "public_core_obligation_installment_events_rollup_view") }} as s
