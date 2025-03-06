{{ config(materialized='table') }}

with collateral_updated as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(json_value(event, '$.audit_info.audit_entry_id') as {{ dbt.type_int() }}) as audit_entry_id,
            cast(json_value(event, '$.abs_diff') as {{ dbt.type_numeric() }}) as abs_diff,
            cast(json_value(event, '$.total_collateral') as {{ dbt.type_numeric() }}) as total_collateral,
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
            ) as recorded_in_ledger_at,
            json_value(event, '$.action') as action
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at,
            event_type,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'audit_info.audit_entry_id') as {{ dbt.type_int() }}) as audit_entry_id,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'abs_diff') as {{ dbt.type_numeric() }}) as abs_diff,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'total_collateral') as {{ dbt.type_numeric() }}) as total_collateral,
            coalesce(cast(
                TO_CHAR(
                    TO_TIMESTAMP(
                        JSON_EXTRACT_PATH_TEXT(event, 'recorded_in_ledger_at')
                    ), 'YYYYMMDD'
                ) as {{ dbt.type_int() }}
            ), 19000101) as recorded_in_ledger_at_date_key,
            TO_TIMESTAMP(
                JSON_EXTRACT_PATH_TEXT(event, 'recorded_in_ledger_at')
            ) as recorded_in_ledger_at,
            JSON_EXTRACT_PATH_TEXT(event, 'action') as action
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where
        cfe.event_type = 'collateral_updated'
        {% if target.type == 'bigquery' %}
            and json_value(event, '$.tx_id') is not null
        {% elif target.type == 'snowflake' %}
            and JSON_EXTRACT_PATH_TEXT(event, 'tx_id') is not null
        {% endif %}

),

collateralization_changed as (

    select
        id as event_id,
        {% if target.type == 'bigquery' %}
            cast(format_date('%Y%m%d', recorded_at) as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at, cast(json_value(event, '$.audit_info.audit_entry_id') as {{ dbt.type_int() }}) as audit_entry_id,
            cast(json_value(event, '$.collateral') as {{ dbt.type_numeric() }}) as collateral,
            cast(json_value(event, '$.price') as {{ dbt.type_numeric() }}) as price,
            cast(json_value(event, '$.outstanding.disbursed') as {{ dbt.type_numeric() }}) as outstanding_disbursed,
            cast(json_value(event, '$.outstanding.interest') as {{ dbt.type_numeric() }}) as outstanding_interest,
            coalesce(cast(
                format_date(
                    '%Y%m%d',
                    parse_timestamp(
                        '%Y-%m-%dT%H:%M:%E*SZ',
                        json_value(event, '$.recorded_at'),
                        'UTC'
                    )
                ) as {{ dbt.type_int() }}
            ), 19000101) as event_recorded_at_date_key,
            parse_timestamp(
                '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.recorded_at'), 'UTC'
            ) as event_recorded_at,
            json_value(event, '$.state') as state
        {% elif target.type == 'snowflake' %}
            cast(TO_CHAR(recorded_at, 'YYYYMMDD') as {{ dbt.type_int() }}) as recorded_at_date_key,
            recorded_at, cast(JSON_EXTRACT_PATH_TEXT(event, 'audit_info.audit_entry_id') as {{ dbt.type_int() }}) as audit_entry_id,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'collateral') as {{ dbt.type_numeric() }}) as collateral,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'price') as {{ dbt.type_numeric() }}) as price,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'outstanding.disbursed') as {{ dbt.type_numeric() }}) as outstanding_disbursed,
            cast(JSON_EXTRACT_PATH_TEXT(event, 'outstanding.interest') as {{ dbt.type_numeric() }}) as outstanding_interest,
            coalesce(cast(
                TO_CHAR(
                    TO_TIMESTAMP(
                        JSON_EXTRACT_PATH_TEXT(event, 'recorded_at')
                    ), 'YYYYMMDD'
                ) as {{ dbt.type_int() }}
            ), 19000101) as event_recorded_at_date_key,
            TO_TIMESTAMP(
                JSON_EXTRACT_PATH_TEXT(event, 'recorded_at')
            ) as event_recorded_at,
            JSON_EXTRACT_PATH_TEXT(event, 'state') as state
        {% endif %}
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'collateralization_changed'

),

btc_price as (

    select last_price_usd
    from {{ ref('stg_bitfinex_ticker_price') }}
    order by requested_at desc
    limit 1

)


select
    cu.event_id,
    cu.recorded_at_date_key,
    cu.recorded_at,
    cu.event_type,
    cu.audit_entry_id,
    cu.recorded_in_ledger_at_date_key,
    cu.recorded_in_ledger_at,
    cu.action,

    cc.event_recorded_at as collateralization_changed_event_recorded_at,
    state as collateralization_changed_state,
    cu.total_collateral,

    cc.price,
    cc.event_recorded_at_date_key as collateralization_changed_event_recorded_at_date_key,

    case
        when lower(action) = 'add' then cu.abs_diff else -cu.abs_diff end as diff,
    coalesce(cc.collateral, 0) as collateral,
    coalesce(cc.outstanding_disbursed, 0) as outstanding_disbursed,
    coalesce(cc.outstanding_interest, 0) as outstanding_interest,

    ((total_collateral / 100000000.0) * (cc.price / 100.0)) as initial_collateral_value_usd,

    ((total_collateral / 100000000.0) * (select last_price_usd from btc_price)) as total_collateral_value_usd,
    (select last_price_usd from btc_price) as last_btc_price_usd
from collateral_updated as cu
left join
    collateralization_changed as cc
    on cu.event_id = cc.event_id and cu.audit_entry_id = cc.audit_entry_id
