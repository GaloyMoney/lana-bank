{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_customer_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            applicant_id,
            customer_type,
            kyc_verification,
            level,
            party_id,
            public_id,
            is_kyc_approved,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_customer_events_rollup") }}
    )
select
    id as customer_id,
    version,
    created_at,
    modified_at,
    applicant_id,
    customer_type,
    kyc_verification,
    level,
    party_id,
    public_id,
    is_kyc_approved,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_customer_events_rollup
