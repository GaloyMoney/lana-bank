with customer as (
    select
        id as customer_id,
        created_at as customer_created_at,
        modified_at as customer_modified_at,

        * except(
            id,
            created_at,
            modified_at,

            last_sequence,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_customer_events_rollup') }}
)


select * from customer
