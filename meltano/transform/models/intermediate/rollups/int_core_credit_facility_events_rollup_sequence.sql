{{ config(
    materialized = 'incremental',
    unique_key = ['credit_facility_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_credit_facility_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (credit_facility_id, version)
        where t.credit_facility_id is null
    {% endif %}
),

proposal as (
    select
        pending_credit_facility_id,
        prop.approval_process_id,
        pend.approval_process_id as pending_approval_process_id,
        is_approval_process_concluded,
        is_approval_process_concluded as approved
    from {{ ref('stg_core_credit_facility_proposal_events_rollup') }} as prop
    left join {{ ref('stg_core_pending_credit_facility_events_rollup') }} as pend
        using (credit_facility_proposal_id, version)
    where is_completed = true
),

transformed as (
    select
        credit_facility_id,
        version,
        customer_id,

        cast(amount as numeric) / {{ var('cents_per_usd') }} as facility_amount_usd,
        cast(json_value(terms, "$.annual_rate") as numeric) as annual_rate,
        cast(json_value(terms, "$.one_time_fee_rate") as numeric) as one_time_fee_rate,

        cast(json_value(terms, "$.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(terms, "$.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(terms, "$.margin_call_cvl") as numeric) as margin_call_cvl,

        cast(json_value(terms, "$.duration.value") as integer) as duration_value,
        json_value(terms, "$.duration.type") as duration_type,

        json_value(terms, "$.accrual_interval.type") as accrual_interval,
        json_value(terms, "$.accrual_cycle_interval.type") as accrual_cycle_interval,

        cast(collateral as numeric) as collateral_amount_sats,
        cast(collateral as numeric) / {{ var('sats_per_bitcoin') }} as collateral_amount_btc,
        cast(price as numeric) / {{ var('cents_per_usd') }} as price_usd_per_btc,
        cast(collateral as numeric)
        / {{ var('sats_per_bitcoin') }}
        * cast(price as numeric)
        / {{ var('cents_per_usd') }} as collateral_amount_usd,
        -- cast(collateralization_ratio as numeric) as collateralization_ratio,
        collateralization_state,

        approval_process_id,
        approved,

        is_approval_process_concluded,
        case when activated_at is not null then true else false end as is_activated,
        cast(activated_at as timestamp) as credit_facility_activated_at,
        is_completed,

        interest_accrual_cycle_idx,
        parse_timestamp("%Y-%m-%dT%H:%M:%E*SZ", json_value(interest_period, "$.start"))
            as interest_period_start_at,
        parse_timestamp("%Y-%m-%dT%H:%M:%E*SZ", json_value(interest_period, "$.end"))
            as interest_period_end_at,
        json_value(interest_period, "$.interval.type") as interest_period_interval_type,

        cast(json_value(outstanding, "$.interest") as numeric)
        / {{ var('cents_per_usd') }} as outstanding_interest_usd,
        cast(json_value(outstanding, "$.disbursed") as numeric)
        / {{ var('cents_per_usd') }} as outstanding_disbursed_usd,

        cast(json_value(terms, "$.interest_due_duration_from_accrual.value") as integer)
            as interest_due_duration_from_accrual_value,
        json_value(terms, "$.interest_due_duration_from_accrual.type")
            as interest_due_duration_from_accrual_type,

        cast(json_value(terms, "$.obligation_overdue_duration_from_due.value") as integer)
            as obligation_overdue_duration_from_due_value,
        json_value(terms, "$.obligation_overdue_duration_from_due.type")
            as obligation_overdue_duration_from_due_type,

        cast(json_value(terms, "$.obligation_liquidation_duration_from_due.value") as integer)
            as obligation_liquidation_duration_from_due_value,
        json_value(terms, "$.obligation_liquidation_duration_from_due.type")
            as obligation_liquidation_duration_from_due_type,
        created_at as credit_facility_created_at,
        modified_at as credit_facility_modified_at,

        json_value(account_ids, "$.facility_account_id") as facility_account_id,
        json_value(account_ids, "$.collateral_account_id") as collateral_account_id,
        json_value(account_ids, "$.fee_income_account_id") as fee_income_account_id,
        json_value(account_ids, "$.interest_income_account_id") as interest_income_account_id,
        json_value(account_ids, "$.interest_defaulted_account_id") as interest_defaulted_account_id,
        json_value(account_ids, "$.disbursed_defaulted_account_id")
            as disbursed_defaulted_account_id,
        json_value(account_ids, "$.interest_receivable_due_account_id")
            as interest_receivable_due_account_id,
        json_value(account_ids, "$.disbursed_receivable_due_account_id")
            as disbursed_receivable_due_account_id,
        json_value(account_ids, "$.interest_receivable_overdue_account_id")
            as interest_receivable_overdue_account_id,
        json_value(account_ids, "$.disbursed_receivable_overdue_account_id")
            as disbursed_receivable_overdue_account_id,
        json_value(account_ids, "$.interest_receivable_not_yet_due_account_id")
            as interest_receivable_not_yet_due_account_id,
        json_value(account_ids, "$.disbursed_receivable_not_yet_due_account_id")
            as disbursed_receivable_not_yet_due_account_id,

        * except (
            credit_facility_id,
            version,
            customer_id,
            amount,
            ledger_tx_ids,
            account_ids,
            terms,
            collateral,
            price,
            -- collateralization_ratio,
            collateralization_state,
            approval_process_id,
            approved,
            is_approval_process_concluded,
            activated_at,
            is_completed,
            interest_accrual_cycle_idx,
            interest_period,
            outstanding,
            created_at,
            modified_at,

            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from source
    left join proposal using (pending_credit_facility_id)
),

final as (
    select
        *,
        collateral_amount_usd / facility_amount_usd * 100 as current_facility_cvl,
        case
            when
                duration_type = "months"
                then
                    timestamp_add(date(credit_facility_activated_at), interval duration_value month)
        end as credit_facility_maturity_at
    from transformed
)

select * from final
