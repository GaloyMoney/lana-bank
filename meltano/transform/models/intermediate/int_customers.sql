select distinct
    json_value(parsed_event.id) as customer_id,
    json_value(parsed_event.email) as email,
    json_value(parsed_event.account_ids.on_balance_sheet_deposit_account_id)
        as on_balance_sheet_deposit_account_id,
    json_value(parsed_event.ipCountry) as ip_country,
    json_value(parsed_event.info.country) as country

from {{ ref('stg_customer_events') }}

where event_type = "initialized"
