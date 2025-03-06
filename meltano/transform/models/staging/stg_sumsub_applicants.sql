select
    customer_id,
    recorded_at,
    content,
    safe.parse_json(content) as parsed_content

from {{ source("lana", "sumsub_applicants") }}
