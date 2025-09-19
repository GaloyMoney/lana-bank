select
    journal_id,
    account_id,
    currency,
    version,
    recorded_at,
    values

from {{ source("lana", "public_cala_balance_history_view") }}
