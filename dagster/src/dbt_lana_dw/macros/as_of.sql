{% macro as_of_timestamp() %}
    {% if var("as_of_date", "") %}
        timestamp('{{ var("as_of_date") }} 23:59:59', 'UTC')
    {% else %}
        timestamp(current_date('UTC'), 'UTC')
    {% endif %}
{% endmacro %}

{% macro as_of_date() %}
    {% if var("as_of_date", "") %}
        date('{{ var("as_of_date") }}')
    {% else %}
        current_date('UTC')
    {% endif %}
{% endmacro %}
