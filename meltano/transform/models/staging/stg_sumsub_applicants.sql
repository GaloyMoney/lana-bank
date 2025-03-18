select
    customer_id,
    recorded_at,
    content,
{% if target.type == 'bigquery' %}
    safe.parse_json(content) as parsed_content
{% elif target.type == 'snowflake' %}
    TRY_PARSE_JSON(content) as parsed_content
{% endif %}

from {{ source("lana", "sumsub_applicants") }}
