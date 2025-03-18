with collateral_updates as (

    select
        id as credit_facility_id,
        recorded_at,
    {% if target.type == 'bigquery' %}
        lax_int64(parsed_event.total_collateral)
        / {{ var('sats_per_bitcoin') }}
            as total_collateral_btc,
        json_value(parsed_event.audit_info.audit_entry_id) as audit_entry_id
    {% elif target.type == 'snowflake' %}
        cast(JSON_EXTRACT_PATH_TEXT(event, 'total_collateral') as {{ dbt.type_int() }})
        / {{ var('sats_per_bitcoin') }}
            as total_collateral_btc,
        JSON_EXTRACT_PATH_TEXT(event, 'audit_info.audit_entry_id') as audit_entry_id
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'collateral_updated'

),

collateralization as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        lax_int64(parsed_event.price) / 100 as initial_price_usd_per_btc,
        json_value(parsed_event.audit_info.audit_entry_id) as audit_entry_id
    {% elif target.type == 'snowflake' %}
        cast(JSON_EXTRACT_PATH_TEXT(event, 'price') as {{ dbt.type_int() }}) / 100 as initial_price_usd_per_btc,
        JSON_EXTRACT_PATH_TEXT(event, 'audit_info.audit_entry_id') as audit_entry_id
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'collateralization_changed'

)

select
    credit_facility_id,
    date(recorded_at) as day,
    any_value(initial_price_usd_per_btc) as initial_price_usd_per_btc,
{% if target.type == 'bigquery' %}
    any_value(total_collateral_btc having max recorded_at) as total_collateral_btc
{% elif target.type == 'snowflake' %}
    GET(MAX_BY(total_collateral_btc, recorded_at, 1), 0) as total_collateral_btc
{% endif %}

from collateral_updates
left join collateralization using (credit_facility_id, audit_entry_id)

group by credit_facility_id, day
