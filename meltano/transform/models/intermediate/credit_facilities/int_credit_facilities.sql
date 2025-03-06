{{ config(materialized='table') }}

with initialized as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% endif %}
        recorded_at,
        event_type,
        {% if target.type == 'bigquery' %}
            cast(json_value(event, '$.terms.annual_rate') as {{ dbt.type_numeric() }}) as terms_annual_rate,
            cast(json_value(event, '$.terms.duration.value') as {{ dbt.type_int() }}) as terms_duration_value,
            cast(json_value(event, '$.terms.initial_cvl') as {{ dbt.type_numeric() }}) as terms_initial_cvl,
            cast(json_value(event, '$.terms.liquidation_cvl') as {{ dbt.type_numeric() }}) as terms_liquidation_cvl,
            cast(json_value(event, '$.terms.margin_call_cvl') as {{ dbt.type_numeric() }}) as terms_margin_call_cvl,
            cast(json_value(event, '$.facility') as {{ dbt.type_numeric() }}) as facility,
            json_value(event, '$.customer_id') as customer_id,
            json_value(event, '$.terms.accrual_interval.type') as terms_accrual_interval_type,
            json_value(event, '$.terms.duration.type') as terms_duration_type,
            json_value(event, '$.terms.incurrence_interval.type') as terms_incurrence_interval_type
        {% elif target.type == 'snowflake' %}
            cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.annual_rate') as {{ dbt.type_numeric() }}) as terms_annual_rate,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.duration.value') as {{ dbt.type_int() }}) as terms_duration_value,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.initial_cvl') as {{ dbt.type_numeric() }}) as terms_initial_cvl,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.liquidation_cvl') as {{ dbt.type_numeric() }}) as terms_liquidation_cvl,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.margin_call_cvl') as {{ dbt.type_numeric() }}) as terms_margin_call_cvl,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'facility') as {{ dbt.type_numeric() }}) as facility,
            JSON_EXTRACT_PATH_TEXT(event, 'customer_id') as customer_id,
            JSON_EXTRACT_PATH_TEXT(event, 'terms.accrual_interval.type') as terms_accrual_interval_type,
            JSON_EXTRACT_PATH_TEXT(event, 'terms.duration.type') as terms_duration_type,
            JSON_EXTRACT_PATH_TEXT(event, 'terms.incurrence_interval.type') as terms_incurrence_interval_type
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'initialized'

),

approval_process_started as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% endif %}
        recorded_at
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'approval_process_started'

),

approval_process_concluded as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            cast(json_value(event, '$.approved') as {{ dbt.type_boolean() }}) as approved
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'approved') as {{ dbt.type_boolean() }}) as approved
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'approval_process_concluded'

),

activated as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% endif %}
        recorded_at,
        {% if target.type == 'bigquery' %}
            cast(
                format_date(
                    '%Y%m%d',
                    parse_timestamp(
                        '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.activated_at')
                    ),
                    'UTC'
                ) as {{ dbt.type_int() }}
            ) as activated_at_date_key,
            parse_timestamp('%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.activated_at'), 'UTC') as activated_at
        {% elif target.type == 'snowflake' %}
            cast(
                TO_CHAR(
                    TO_TIMESTAMP(
                        JSON_EXTRACT_PATH_TEXT(event, 'activated_at')
                    ), 'YYYYMMDD'
                ) as {{ dbt.type_int() }}
            ) as activated_at_date_key,
            TO_TIMESTAMP(JSON_EXTRACT_PATH_TEXT(event, 'activated_at')) as activated_at
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'activated'

),

completed as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
        {% endif %}
        recorded_at,
        {% if target.type == 'bigquery' %}
            cast(
                format_date(
                    '%Y%m%d',
                    parse_timestamp(
                        '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.completed_at')
                    ),
                    'UTC'
                ) as {{ dbt.type_int() }}
            ) as completed_at_date_key,
            parse_timestamp('%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.completed_at'), 'UTC') as completed_at
        {% elif target.type == 'snowflake' %}
            cast(
                TO_CHAR(
                    TO_TIMESTAMP(
                        JSON_EXTRACT_PATH_TEXT(event, 'completed_at')
                    ), 'YYYYMMDD'
                ) as {{ dbt.type_int() }}
            ) as completed_at_date_key,
            TO_TIMESTAMP(JSON_EXTRACT_PATH_TEXT(event, 'completed_at')) as completed_at
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'completed'

)


select
    i.event_id,
    i.recorded_at_date_key,
    i.recorded_at,
    i.event_type,
    i.terms_annual_rate,
    i.terms_duration_value,
    i.terms_initial_cvl,
    i.terms_liquidation_cvl,
    i.terms_margin_call_cvl,
    i.customer_id,
    i.terms_accrual_interval_type,
    i.terms_duration_type,
    i.terms_incurrence_interval_type,

    aps.recorded_at as approval_process_started_recorded_at,
    apc.recorded_at as approval_process_concluded_recorded_at,

    a.recorded_at as activated_recorded_at,
    a.activated_at,
    c.recorded_at as completed_recorded_at,

    c.completed_at,
    i.facility,
    coalesce(aps.recorded_at_date_key, 19000101) as approval_process_started_recorded_at_date_key,
    coalesce(apc.recorded_at_date_key, 19000101) as approval_process_concluded_recorded_at_date_key,

    coalesce(apc.approved, false) as approval_process_concluded_approved,
    coalesce(a.recorded_at_date_key, 19000101) as activated_recorded_at_date_key,
    coalesce(a.activated_at_date_key, 19000101) as activated_at_date_key,
    coalesce(c.recorded_at_date_key, 19000101) as completed_recorded_at_date_key,

    coalesce(c.completed_at_date_key, 19000101) as completed_at_date_key
from initialized as i
left join approval_process_started as aps on i.event_id = aps.event_id
left join approval_process_concluded as apc on i.event_id = apc.event_id
left join activated as a on i.event_id = a.event_id
left join completed as c on i.event_id = c.event_id
