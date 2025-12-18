{% test accepted_range(model, column_name, min_value, max_value) %}
select *
from {{ model }}
where {{ adapter.quote(column_name) }} is not null
  and {{ adapter.quote(column_name) }} not between {{ min_value }} and {{ max_value }}
{% endtest %}
