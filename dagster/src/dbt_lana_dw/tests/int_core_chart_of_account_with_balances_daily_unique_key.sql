with
    duplicates as (
        select as_of_date, code, count(*) as duplicate_count
        from {{ ref("int_core_chart_of_account_with_balances_daily") }}
        group by as_of_date, code
        having count(*) > 1
    )

select *
from duplicates
