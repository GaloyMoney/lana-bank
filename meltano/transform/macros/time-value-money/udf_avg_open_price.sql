{% macro create_udf_avg_open_price() %}

CREATE OR REPLACE FUNCTION {{target.schema}}.udf_avg_open_price (
{% if target.type == 'bigquery' %}
	DEPOSIT ARRAY<{{ dbt.type_float() }}>,
	OPEN_PRICE ARRAY<{{ dbt.type_float() }}>
) RETURNS ARRAY<{{ dbt.type_float() }}>
LANGUAGE js
AS r"""
{% elif target.type == 'snowflake' %}
	DEPOSIT ARRAY,
	OPEN_PRICE ARRAY
) RETURNS ARRAY
LANGUAGE JAVASCRIPT
AS $$
{% endif %}
	var avg_open_price = [];
	var balance = 0.0;
	var current_avg_open_price = 0.0;
	for (var i = 0; i < DEPOSIT.length; i++) {
		balance = balance + DEPOSIT[i];
		if (DEPOSIT[i] > 0) {
			current_avg_open_price = ((balance-DEPOSIT[i])*current_avg_open_price + DEPOSIT[i]*OPEN_PRICE[i]) / balance;
		}
		avg_open_price.push(current_avg_open_price);
	}
	return avg_open_price;
{% if target.type == 'bigquery' %}
"""
{% elif target.type == 'snowflake' %}
$$
{% endif %}

{% endmacro %}
