{% macro as_of_timestamp() %}
    timestamp('{{ var("as_of_date") }} 23:59:59', 'UTC')
{% endmacro %}

{% macro as_of_date() %} date('{{ var("as_of_date") }}') {% endmacro %}
