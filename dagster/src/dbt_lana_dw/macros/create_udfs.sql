{% macro create_udfs() %}

{% do run_query(create_udf_json_array_to_code()) %}

{% endmacro %}
