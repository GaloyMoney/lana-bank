{{
    config(
        unique_key=["party_id", "version"],
    )
}}


with
    source as (select s.* from {{ ref("stg_core_party_events_rollup") }} as s),

    transformed as (
        select
            * except (party_id, created_at, modified_at),
            party_id,
            created_at as party_created_at,

            modified_at as party_modified_at
        from source
    )

select *
from transformed
