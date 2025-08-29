select *
from {{ ref('int_payment_history') }}
{# order by activated_at #}
