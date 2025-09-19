with ordered as (

    select
        customer_id,
        recorded_at,
        content,
        row_number()
            over (
                partition by customer_id
                order by recorded_at desc
            )
            as order_recorded_at_desc

    from {{ source("lana", "sumsub_applicants_view") }}

)

select
    * except (order_recorded_at_desc),
    safe.parse_json(content) as parsed_content

from ordered

where order_recorded_at_desc = 1
