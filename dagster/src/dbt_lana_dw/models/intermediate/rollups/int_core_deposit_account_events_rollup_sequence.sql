with
    source as (
        select
            deposit_account_id,
            version,
            created_at,
            modified_at,
            account_holder_id,
            account_ids,
            public_id,
            status,
            loaded_to_dw_at
        from {{ ref("stg_core_deposit_account_events_rollup") }} as s
    ),
    transformed as (
        select
            deposit_account_id,
            account_holder_id as customer_id,
            created_at as deposit_account_created_at,
            modified_at as deposit_account_modified_at,

            version,
            account_ids,
            public_id,
            status,
            loaded_to_dw_at
        from source
    )

select
    deposit_account_id,
    customer_id,
    deposit_account_created_at,
    deposit_account_modified_at,
    version,
    account_ids,
    public_id,
    status,
    loaded_to_dw_at
from transformed
