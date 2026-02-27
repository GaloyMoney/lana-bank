with
    chart_row_count as (
        select count(*) as expected_rows
        from {{ ref("int_core_chart_of_accounts") }}
    ),

    daily_row_counts as (
        select as_of_date, count(*) as actual_rows
        from {{ ref("int_core_chart_of_account_with_balances_daily") }}
        group by as_of_date
    ),

    failures as (
        select
            daily_row_counts.as_of_date,
            chart_row_count.expected_rows,
            daily_row_counts.actual_rows
        from daily_row_counts
        cross join chart_row_count
        where daily_row_counts.actual_rows != chart_row_count.expected_rows
    )

select *
from failures
