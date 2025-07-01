with customer as (
    select
        id as customer_id,

        * except(
            id,

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
