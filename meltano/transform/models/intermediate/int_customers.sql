select distinct
{% if target.type == 'bigquery' %}
    json_value(parsed_event.id) as customer_id,
    json_value(parsed_event.email) as email,
    json_value(parsed_event.account_ids.on_balance_sheet_deposit_account_id) as on_balance_sheet_deposit_account_id,
    json_value(parsed_event.ipcountry) as ip_country,
    json_value(parsed_event.info.country) as country
{% elif target.type == 'snowflake' %}
    TO_VARCHAR(parsed_event:id) as customer_id,
    TO_VARCHAR(parsed_event:email) as email,
    TO_VARCHAR(parsed_event:account_ids.on_balance_sheet_deposit_account_id) as on_balance_sheet_deposit_account_id,
    TO_VARCHAR(parsed_event:ipcountry) as ip_country,
    TO_VARCHAR(parsed_event:info.country) as country
{% endif %}

from {{ ref('stg_customer_events') }}

where event_type = 'initialized'
