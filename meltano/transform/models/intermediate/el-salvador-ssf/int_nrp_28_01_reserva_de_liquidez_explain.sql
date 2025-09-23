with

config as (
    select
        order_by,
        account_code,
        title,
        eng_title,
        account_name,
        eng_account_name,
        coalesce(coefficient, 1) as coefficient
    from {{ ref('static_nrp_28_01_account_config') }}
    left join unnest(sum_account_codes) as account_code
    left join {{ ref('static_nrp_28_01_liquidity_coefficients') }} using (account_code)

    union all

    select
        order_by,
        account_code,
        title,
        eng_title,
        account_name,
        eng_account_name,
        -1 * coalesce(coefficient, 1) as coefficient
    from {{ ref('static_nrp_28_01_account_config') }}
    left join unnest(diff_account_codes) as account_code
    left join {{ ref('static_nrp_28_01_liquidity_coefficients') }} using (account_code)
),

chart as (
    select *
    from {{ ref('int_core_chart_of_account_with_balances') }}
),

final as (
    select
        config.*,
        chart.* except (balance),
        coalesce(balance, 0) as balance
    from config
    left join chart on code = account_code
)

select *
from final
