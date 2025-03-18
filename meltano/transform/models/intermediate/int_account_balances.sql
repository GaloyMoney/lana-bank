{% if target.type == 'bigquery' %}
    with stg_account_balances as(
        select
            cast(
                json_value(
                    any_value(values having max recorded_at), '$.settled.cr_balance'
                ) as {{ dbt.type_numeric() }}
            ) as settled_cr,
            cast(
                json_value(
                    any_value(values having max recorded_at), '$.settled.dr_balance'
                ) as {{ dbt.type_numeric() }}
            ) as settled_dr,
            json_value(values, '$.account_id') as account_id,
            json_value(values, '$.currency') as currency

        from {{ ref('stg_account_balances') }}
    )

    select
        any_value(settled_cr),
        any_value(settled_dr),
        account_id,
        currency,

    from stg_account_balances
    group by account_id, currency
{% elif target.type == 'snowflake' %}
    with stg_account_balances_latest_values as(
        select
            account_id,
            currency,
            GET(MAX_BY("VALUES", recorded_at, 1), 0) as latest_values,

        from {{ ref('stg_account_balances') }}
        group by account_id, currency
    ), stg_account_balances as(

        select
            cast(
                JSON_EXTRACT_PATH_TEXT(latest_values, 'settled.cr_balance') as {{ dbt.type_numeric() }}
            ) as settled_cr,
            cast(
                JSON_EXTRACT_PATH_TEXT(latest_values, 'settled.dr_balance') as {{ dbt.type_numeric() }}
            ) as settled_dr,
            JSON_EXTRACT_PATH_TEXT(latest_values, 'account_id') as account_id,
            JSON_EXTRACT_PATH_TEXT(latest_values, 'currency') as currency
        from stg_account_balances_latest_values
    )

    select
        settled_cr,
        settled_dr,
        account_id,
        currency,

    from stg_account_balances
{% endif %}
