select *
from {{ ref('int_active_loans') }}
order by credit_facility_activated_at
