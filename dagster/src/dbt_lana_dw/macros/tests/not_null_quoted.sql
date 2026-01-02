{% test not_null_quoted(model, column_name) %}
select *
from {{ model }}
where {{ adapter.quote(column_name) }} is null
{% endtest %}
