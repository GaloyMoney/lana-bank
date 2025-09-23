with all_accounts as (

    select
        id as account_id,
        name as account_name,
        normal_balance_type,
        code as account_code,
        -- TODO: need fixing, where did old latest_values go which held "$.config.is_account_set" flag
        --        lax_bool(
        --            parse_json(json_value(latest_values, "$.config.is_account_set"))
        --        ) as is_account_set
        false as is_account_set

    from {{ ref('stg_accounts') }}
    where _sdc_batched_at >= (
        select coalesce(max(_sdc_batched_at), '1900-01-01')
        from {{ ref('stg_core_chart_node_events') }}
        where event_type = 'initialized'
    )

),

credit_facilities as (

    select distinct
        credit_facility_key,
        facility_account_id,
        collateral_account_id,
        fee_income_account_id,
        interest_income_account_id,
        interest_defaulted_account_id,
        disbursed_defaulted_account_id,
        interest_receivable_due_account_id,
        disbursed_receivable_due_account_id,
        interest_receivable_overdue_account_id,
        disbursed_receivable_overdue_account_id,
        interest_receivable_not_yet_due_account_id,
        disbursed_receivable_not_yet_due_account_id

    from {{ ref('int_approved_credit_facilities') }}

),

credit_facility_accounts as (

    select distinct
        credit_facility_key,
        facility_account_id as account_id,
        'facility_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        collateral_account_id as account_id,
        'collateral_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        fee_income_account_id as account_id,
        'fee_income_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_income_account_id as account_id,
        'interest_income_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_defaulted_account_id as account_id,
        'interest_defaulted_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        disbursed_defaulted_account_id as account_id,
        'disbursed_defaulted_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_receivable_due_account_id as account_id,
        'interest_receivable_due_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        disbursed_receivable_due_account_id as account_id,
        'disbursed_receivable_due_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_receivable_overdue_account_id as account_id,
        'interest_receivable_overdue_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        disbursed_receivable_overdue_account_id as account_id,
        'disbursed_receivable_overdue_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        interest_receivable_not_yet_due_account_id as account_id,
        'interest_receivable_not_yet_due_account' as account_type
    from credit_facilities

    union distinct

    select distinct
        credit_facility_key,
        disbursed_receivable_not_yet_due_account_id as account_id,
        'disbursed_receivable_not_yet_due_account' as account_type
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
    using (account_id)
