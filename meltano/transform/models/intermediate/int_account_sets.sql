select
    id as account_set_id,
    set_name,
{% if target.type == 'bigquery' %}
    row_number() over () as set_key
{% elif target.type == 'snowflake' %}
    row_number() over (order by null) as set_key
{% endif %}

from {{ ref('stg_account_sets') }}
