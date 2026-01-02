{{
    config(
        unique_key=["pending_credit_facility_id", "version"],
    )
}}

with
    source as (
        select s.* from {{ ref("stg_core_pending_credit_facility_events_rollup") }} as s
    ),

    transformed as (
        select
            * except (pending_credit_facility_id, version),
            pending_credit_facility_id,
            version
        from source
    ),

    final as (select * from transformed)

select *
from final
