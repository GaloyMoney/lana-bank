with
    duplicates as (
        select as_of_date, account_id, currency, count(*) as duplicate_count
        from {{ ref("int_account_balances_daily") }}
        group by as_of_date, account_id, currency
        having count(*) > 1
    )

select *
from duplicates
