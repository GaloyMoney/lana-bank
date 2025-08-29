{{ config(
    materialized = 'incremental',
    unique_key = ['customer_id', 'version'],
) }}


with source as (
    select
        s.*
    from {{ ref('stg_core_customer_events_rollup') }} as s

    {% if is_incremental() %}
        left join {{ this }} as t using (customer_id, version)
        where t.customer_id is null
    {% endif %}
)


, transformed as (
    select
        customer_id,
        created_at as customer_created_at,
        modified_at as customer_modified_at,

        * except(
            customer_id,
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
