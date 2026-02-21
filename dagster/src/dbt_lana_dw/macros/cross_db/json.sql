{#
  Cross-platform JSON functions.
  
  BigQuery: json_value(), json_query_array()
  Postgres: ->> operator, jsonb_array_elements_text()
#}

{# Extract a scalar value from JSON #}
{% macro json_value(json_col, path) %}
  {% if target.type == 'bigquery' %}
    json_value({{ json_col }}, '$.{{ path }}')
  {%- elif target.type == 'postgres' %}
    ({{ json_col }}->>'{{ path }}')
  {%- else %}
    ({{ json_col }}->>'{{ path }}')
  {%- endif %}
{% endmacro %}


{# Extract a JSON array for use with unnest/lateral #}
{% macro json_query_array(json_col, path) %}
  {% if target.type == 'bigquery' %}
    json_query_array({{ json_col }}, '$.{{ path }}')
  {%- elif target.type == 'postgres' %}
    jsonb_array_elements({{ json_col }}->'{{ path }}')
  {%- else %}
    jsonb_array_elements({{ json_col }}->'{{ path }}')
  {%- endif %}
{% endmacro %}


{# Convert JSON array of strings to SQL array #}
{# If path is empty, treats the column itself as a JSON array #}
{% macro array_from_json_strings(json_col, path='') %}
  {% if target.type == 'bigquery' %}
    {% if path == '' %}
    array(
      select json_value(item, '$')
      from unnest(json_query_array({{ json_col }}, '$')) as item
    )
    {%- else %}
    array(
      select json_value(item, '$')
      from unnest(json_query_array({{ json_col }}, '$.{{ path }}')) as item
    )
    {%- endif %}
  {%- elif target.type == 'postgres' %}
    {% if path == '' %}
    (
      select array_agg(elem)
      from jsonb_array_elements_text({{ json_col }}::jsonb) as elem
    )
    {%- else %}
    (
      select array_agg(elem)
      from jsonb_array_elements_text(({{ json_col }}::jsonb)->'{{ path }}') as elem
    )
    {%- endif %}
  {%- else %}
    {% if path == '' %}
    (
      select array_agg(elem)
      from jsonb_array_elements_text({{ json_col }}::jsonb) as elem
    )
    {%- else %}
    (
      select array_agg(elem)
      from jsonb_array_elements_text(({{ json_col }}::jsonb)->'{{ path }}') as elem
    )
    {%- endif %}
  {%- endif %}
{% endmacro %}
