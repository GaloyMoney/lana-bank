with
    duplicates as (
        select
            journal_id,
            account_id,
            currency,
            effective,
            version,
            count(*) as duplicate_count
        from {{ ref("stg_cumulative_effective_balances") }}
        group by journal_id, account_id, currency, effective, version
        having count(*) > 1
    )

select *
from duplicates
