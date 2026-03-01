with
    spine_stats as (
        select
            count(*) as total_rows,
            countif(as_of_date = current_date("UTC")) as today_rows
        from {{ ref("int_date_spine") }}
    )

select *
from spine_stats
where total_rows = 0 or today_rows = 0
