with
    source_keys as (
        select distinct journal_id, account_id, currency, effective, version
        from {{ source("lana", "cala_cumulative_effective_balances") }}
    ),

    staged_keys as (
        select distinct journal_id, account_id, currency, effective, version
        from {{ ref("stg_cumulative_effective_balances") }}
    ),

    missing_keys as (
        select source_keys.*
        from source_keys
        left join
            staged_keys
            on source_keys.journal_id = staged_keys.journal_id
            and source_keys.account_id = staged_keys.account_id
            and source_keys.currency = staged_keys.currency
            and source_keys.effective = staged_keys.effective
            and source_keys.version = staged_keys.version
        where staged_keys.journal_id is null
    )

select *
from missing_keys
