{{ config(materialized='table') }}

with payment_recorded as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(json_value(event, '$.disbursal_amount') as {{ dbt.type_numeric() }}) as disbursal_amount,
            cast(json_value(event, '$.interest_amount') as {{ dbt.type_numeric() }}) as interest_amount,
            coalesce(cast(
                format_date(
                    '%Y%m%d',
                    parse_timestamp(
                        '%Y-%m-%dT%H:%M:%E*SZ',
                        json_value(event, '$.recorded_in_ledger_at'),
                        'UTC'
                    )
                ) as {{ dbt.type_int() }}
            ), 19000101) as recorded_in_ledger_at_date_key,
            parse_timestamp(
                '%Y-%m-%dT%H:%M:%E*SZ',
                json_value(event, '$.recorded_in_ledger_at'),
                'UTC'
            ) as recorded_in_ledger_at
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'disbursal_amount') as {{ dbt.type_numeric() }}) as disbursal_amount,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'interest_amount') as {{ dbt.type_numeric() }}) as interest_amount,
            coalesce(cast(
                TO_CHAR(
                    TO_TIMESTAMP(
                        JSON_EXTRACT_PATH_TEXT(event, 'recorded_in_ledger_at')
                    ), 'YYYYMMDD'
                ) as {{ dbt.type_int() }}
            ), 19000101) as recorded_in_ledger_at_date_key,
            TO_TIMESTAMP(
                JSON_EXTRACT_PATH_TEXT(event, 'recorded_in_ledger_at')
            ) as recorded_in_ledger_at
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where
        cfe.event_type = 'payment_recorded'
        {% if target.type == 'bigquery' %}
            and json_value(event, '$.tx_id') is not null
        {% elif target.type == 'snowflake' %}
            and JSON_EXTRACT_PATH_TEXT(event, 'tx_id') is not null
        {% endif %}

)


select *
from payment_recorded
