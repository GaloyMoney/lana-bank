select distinct
    json_value(parsed_event.id) as deposit_id,
    cast(json_value(parsed_event.amount) as numeric) as amount,
    json_value(parsed_event.deposit_account_id) as deposit_account_id,
    json_value(parsed_event.ledger_transaction_id) as ledger_transaction_id,
    recorded_at

from {{ ref('stg_deposit_events') }}

where event_type = "initialized"
