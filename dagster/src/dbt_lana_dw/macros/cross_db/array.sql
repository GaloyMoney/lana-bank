{#
  Cross-platform array functions.
  
  BigQuery: safe_offset (0-indexed), unnest()
  Postgres: array indexing (1-indexed), unnest()
#}

{# Safe array element access (0-indexed input) #}
{% macro array_at(arr, idx) %}
  {% if target.type == 'bigquery' %}
    {{ arr }}[safe_offset({{ idx }})]
  {%- elif target.type == 'postgres' %}
    {{ arr }}[{{ idx }} + 1]
  {%- else %}
    {{ arr }}[{{ idx }} + 1]
  {%- endif %}
{% endmacro %}


{# Split string and get element at index (0-indexed) #}
{% macro split_part_at(str_col, delimiter, idx) %}
  {% if target.type == 'bigquery' %}
    split({{ str_col }}, '{{ delimiter }}')[safe_offset({{ idx }})]
  {%- elif target.type == 'postgres' %}
    split_part({{ str_col }}, '{{ delimiter }}', {{ idx }} + 1)
  {%- else %}
    split_part({{ str_col }}, '{{ delimiter }}', {{ idx }} + 1)
  {%- endif %}
{% endmacro %}


{# Unnest with alias - handles syntax differences #}
{% macro unnest_as(arr_expr, alias) %}
  {% if target.type == 'bigquery' %}
    unnest({{ arr_expr }}) as {{ alias }}
  {%- elif target.type == 'postgres' %}
    unnest({{ arr_expr }}) as {{ alias }}
  {%- else %}
    unnest({{ arr_expr }}) as {{ alias }}
  {%- endif %}
{% endmacro %}
