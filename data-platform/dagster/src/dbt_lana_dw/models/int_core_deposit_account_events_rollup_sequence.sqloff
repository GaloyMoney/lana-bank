{{ config(
    materialized = 'incremental',
    unique_key = ['deposit_account_id', 'version'],
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_deposit_account_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (deposit_account_id, version)
        where t.deposit_account_id is null
    {% endif %}
)


, transformed as (
    select
        deposit_account_id,
        account_holder_id as customer_id,
        created_at as deposit_account_created_at,
        modified_at as deposit_account_modified_at,

        * except(
            deposit_account_id,
            account_holder_id,
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
