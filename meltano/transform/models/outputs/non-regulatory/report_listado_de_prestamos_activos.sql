select *
from {{ ref('int_active_loans') }}
order by activated_at
