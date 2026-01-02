{{
    config(
        unique_key=["liquidation_id", "version"],
    )
}}


with
    source as (select s.* from {{ ref("stg_core_liquidation_events_rollup") }} as s),

    transformed as (
        select
            liquidation_id,
            credit_facility_id,

            {# cast(effective as timestamp) as effective, #}
            is_completed,
            {# cast(initial_amount as numeric) / {{ var('cents_per_usd') }} as initial_amount_usd, #}
            created_at as liquidation_created_at,
            modified_at as liquidation_modified_at,

            * except (
                liquidation_id,
                credit_facility_id,

                {# effective, #}
                is_completed,
                {# initial_amount, #}
                created_at,
                modified_at
            )
        from source
    )

select *
from transformed
