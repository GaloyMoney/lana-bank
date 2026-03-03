with
    daily_today as (
        select account_id, currency, settled_dr, settled_cr
        from {{ ref("int_account_balances_daily") }}
        where
            as_of_date
            = (select max(as_of_date) from {{ ref("int_account_balances_daily") }})
    ),

    current_balances as (
        select account_id, currency, settled_dr, settled_cr
        from {{ ref("int_account_balances") }}
    ),

    failures as (
        select
            coalesce(d.account_id, c.account_id) as account_id,
            coalesce(d.currency, c.currency) as currency,
            d.settled_dr as daily_dr,
            c.settled_dr as current_dr,
            d.settled_cr as daily_cr,
            c.settled_cr as current_cr
        from daily_today as d
        full outer join current_balances as c using (account_id, currency)
        where
            d.settled_dr != c.settled_dr
            or d.settled_cr != c.settled_cr
            or d.account_id is null
            or c.account_id is null
    )

select *
from failures
