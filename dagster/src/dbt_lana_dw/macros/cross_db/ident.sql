{#
  Cross-platform identifier quoting.
  
  BigQuery uses backticks: `column_name`
  Postgres uses double quotes: "column_name"
  
  Usage:
    select {{ ident('primer_apellido') }} from ...
#}

{% macro ident(name) %}
  {% if target.type == 'bigquery' %}
    `{{ name }}`
  {%- elif target.type == 'postgres' %}
    "{{ name }}"
  {%- else %}
    "{{ name }}"
  {%- endif %}
{% endmacro %}
