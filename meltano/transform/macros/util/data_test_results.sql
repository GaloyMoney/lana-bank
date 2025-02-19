{% macro create_data_test_results() %}

create or replace table {{target.schema}}.data_test_results (test string, errors int64);

for test in (
    select table_name
    from {{target.schema}}.INFORMATION_SCHEMA.TABLES
    where table_name like "assert_%"
        or table_name like "accepted_values_%"
        or table_name like "unique_%"
        or table_name like "not_null_%"
) do
  execute immediate format('''
    insert into {{target.schema}}.data_test_results (test, errors)
    select "%s", count(*) from `{{target.schema}}.%s`''',
    test.table_name,  test.table_name);
end for;

{% endmacro %}
