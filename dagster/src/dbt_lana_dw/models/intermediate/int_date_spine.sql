{{ config(materialized="table") }}

with
    chart_initialized_at as (
        select
            coalesce(
                max(recorded_at), timestamp("1900-01-01")
            ) as initialized_recorded_at
        from {{ ref("stg_core_chart_node_events") }}
        where event_type = "initialized"
    ),

    cumulative_effective_balances as (
        select effective
        from {{ ref("stg_cumulative_effective_balances") }}
        where
            effective
            >= date((select initialized_recorded_at from chart_initialized_at))
    ),

    bounds as (
        select coalesce(min(effective), current_date("UTC")) as min_effective_date
        from cumulative_effective_balances
    ),

    final as (
        select as_of_date
        from
            bounds,
            unnest(
                generate_date_array(
                    least(min_effective_date, current_date("UTC")),
                    current_date("UTC"),
                    interval 1 day
                )
            ) as as_of_date
    )

select *
from final
