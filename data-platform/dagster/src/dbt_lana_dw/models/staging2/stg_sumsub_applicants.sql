select
    customer_id,
    recorded_at,
    content,
    content as parsed_content

from {{ source("lana", "sumsub_applicants_view") }}
