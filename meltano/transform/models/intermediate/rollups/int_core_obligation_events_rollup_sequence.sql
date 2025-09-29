{{ config(
    materialized = 'incremental',
    unique_key = ['obligation_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_obligation_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (obligation_id, version)
        where t.obligation_id is null
    {% endif %}
),

transformed as (
    select
        obligation_id,
        credit_facility_id,

        cast(effective as timestamp) as effective,
        obligation_type,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(payment_allocation_amount as numeric)
        / {{ var('cents_per_usd') }} as payment_allocation_amount_usd,
        cast(due_amount as numeric) / {{ var('cents_per_usd') }} as due_amount_usd,
        cast(overdue_amount as numeric) / {{ var('cents_per_usd') }} as overdue_amount_usd,
        cast(defaulted_amount as numeric) / {{ var('cents_per_usd') }} as defaulted_amount_usd,

        current_timestamp() >= cast(due_date as timestamp)
        and current_timestamp() >= cast(overdue_date as timestamp)
        and not is_completed
        and not is_defaulted_recorded
        and cast(amount as numeric) > 0
            as overdue,
        case
            when
                is_completed
                or is_defaulted_recorded
                or cast(amount as numeric) <= 0
                then 0
            else 1
        end
        * greatest(timestamp_diff(current_timestamp(), cast(overdue_date as timestamp), day), 0)
            as overdue_days,

        cast(due_date as timestamp) as due_date,
        cast(overdue_date as timestamp) as overdue_date,
        cast(liquidation_date as timestamp) as liquidation_date,
        cast(defaulted_date as timestamp) as defaulted_date,
        is_due_recorded,
        is_overdue_recorded,
        is_defaulted_recorded,
        is_completed,
        created_at as obligation_created_at,
        modified_at as obligation_modified_at,

        * except (
            obligation_id,
            credit_facility_id,

            effective,
            obligation_type,
            amount,
            payment_allocation_amount,
            due_amount,
            overdue_amount,
            defaulted_amount,
            due_date,
            overdue_date,
            liquidation_date,
            defaulted_date,
            is_due_recorded,
            is_overdue_recorded,
            is_defaulted_recorded,
            is_completed,
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
)

select * from transformed
