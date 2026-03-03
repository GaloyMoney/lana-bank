with
    account_date_bounds as (
        select
            account_id,
            currency,
            min(as_of_date) as first_date,
            max(as_of_date) as last_date,
            count(*) as actual_days
        from {{ ref("int_account_balances_daily") }}
        group by account_id, currency
    ),

    failures as (
        select *, date_diff(last_date, first_date, day) + 1 as expected_days
        from account_date_bounds
        where actual_days != date_diff(last_date, first_date, day) + 1
    )

select *
from failures
