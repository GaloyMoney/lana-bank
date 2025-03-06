{{ config(materialized='table') }}

with disbursal_initiated as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(json_value(event, '$.amount') as {{ dbt.type_numeric() }}) as amount,
            cast(json_value(event, '$.idx') as {{ dbt.type_int() }}) as idx
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'amount') as {{ dbt.type_numeric() }}) as amount,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'idx') as {{ dbt.type_int() }}) as idx
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'disbursal_initiated'

),

disbursal_concluded as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            cast(
                format_date(
                    '%Y%m%d',
                    parse_timestamp(
                        '%Y-%m-%dT%H:%M:%E*SZ',
                        json_value(event, '$.recorded_at'),
                        'UTC'
                    )
                ) as {{ dbt.type_int() }}
            ) as event_recorded_at_date_key,
            cast(json_value(event, '$.idx') as {{ dbt.type_int() }}) as idx,
            parse_timestamp(
                '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.recorded_at'), 'UTC'
            ) as event_recorded_at
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            cast(
                TO_CHAR(TO_TIMESTAMP(JSON_EXTRACT_PATH_TEXT(event, 'recorded_at')), 'YYYYMMDD') as {{ dbt.type_int() }}
            ) as event_recorded_at_date_key,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'idx') as {{ dbt.type_int() }}) as idx,
            TO_TIMESTAMP(JSON_EXTRACT_PATH_TEXT(event, 'recorded_at')) as event_recorded_at
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where
        cfe.event_type = 'disbursal_concluded'
        {% if target.type == 'bigquery' %}
            and json_value(event, '$.tx_id') is not null
        {% elif target.type == 'snowflake' %}
            and JSON_EXTRACT_PATH_TEXT(event, 'tx_id') is not null
        {% endif %}

)


select
    di.event_id,
    di.recorded_at_date_key,
    di.recorded_at,
    di.event_type,
    di.idx,

    dc.event_recorded_at as disbursal_concluded_event_recorded_at,
    di.amount,

    coalesce(dc.event_recorded_at_date_key, 19000101) as disbursal_concluded_event_recorded_at_date_key
from disbursal_initiated as di
left join
    disbursal_concluded as dc
    on di.event_id = dc.event_id and di.idx = dc.idx
