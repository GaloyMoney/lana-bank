{{
    config(
        unique_key=["credit_facility_proposal_id", "version"],
    )
}}

with
    source as (
        select s.*
        from {{ ref("stg_core_credit_facility_proposal_events_rollup") }} as s
    ),

    transformed as (
        select
            * except (credit_facility_proposal_id, version),
            credit_facility_proposal_id,
            version
        from source
    ),

    final as (select * from transformed)

select *
from final
