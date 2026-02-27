with
    duplicates as (
        select as_of_date, account_set_id, member_id, count(*) as duplicate_count
        from {{ ref("int_account_sets_expanded_with_balances_daily") }}
        group by as_of_date, account_set_id, member_id
        having count(*) > 1
    )

select *
from duplicates
