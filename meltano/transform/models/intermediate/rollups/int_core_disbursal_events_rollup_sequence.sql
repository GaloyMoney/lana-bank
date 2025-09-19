{{ config(
    materialized = 'incremental',
    unique_key = ['disbursal_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_disbursal_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (disbursal_id, version)
        where t.disbursal_id is null
    {% endif %}
),

transformed as (
    select
        disbursal_id,
        credit_facility_id,
        cast(effective as timestamp) as effective,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        approved,
        is_approval_process_concluded,
        is_settled,
        is_cancelled,
        cast(due_date as timestamp) as due_date,
        cast(overdue_date as timestamp) as overdue_date,
        cast(liquidation_date as timestamp) as liquidation_date,
        created_at as disbursal_created_at,
        modified_at as disbursal_modified_at,

        * except (
            disbursal_id,
            credit_facility_id,

            effective,
            amount,
            approved,
            is_approval_process_concluded,
            is_settled,
            is_cancelled,
            due_date,
            overdue_date,
            liquidation_date,
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
