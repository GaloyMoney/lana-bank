with all_accounts as (

    select
        id as account_id,
        name as account_name,
        normal_balance_type,
        code as account_code,
        lax_bool(
            parse_json(json_value(latest_values, "$.config.is_account_set"))
        ) as is_account_set

    from {{ ref('stg_accounts') }}

),

credit_facilities as (

    select distinct
        credit_facility_key,
        collateral_account_id,
        disbursed_receivable_account_id,
        facility_account_id,
        fee_income_account_id,
        interest_account_id,
        interest_receivable_account_id

    from {{ ref('int_approved_credit_facilities') }}

),

credit_facility_accounts as (

    select distinct
        credit_facility_key,
        collateral_account_id as account_id,
        "collateral_account" as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        disbursed_receivable_account_id as account_id,
        "disbursed_receivable_account" as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        facility_account_id as account_id,
        "facility_account" as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        fee_income_account_id as account_id,
        "fee_income_account" as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_account_id as account_id,
        "interest_account" as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_receivable_account_id as account_id,
        "interest_receivable_account" as account_type
    from credit_facilities

)

select
    account_id,
    account_name,
    normal_balance_type,
    account_code,
    is_account_set,
    credit_facility_key,
    account_type,
    row_number() over () as account_key

from all_accounts
left join
    credit_facility_accounts
    on all_accounts.account_id = credit_facility_accounts.account_id
