with
    all_accounts as (

        select
            id as account_id,
            name as account_name,
            normal_balance_type,
            code as account_code

        from {{ ref("stg_accounts") }}
        where
            loaded_to_dw_at >= (
                select coalesce(max(loaded_to_dw_at), '1900-01-01')
                from {{ ref("stg_core_chart_node_events") }}
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

        from {{ ref("int_approved_credit_facilities") }}

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
    credit_facility_key,
    account_type,
    row_number() over () as account_key

from all_accounts
left join credit_facility_accounts using (account_id)
