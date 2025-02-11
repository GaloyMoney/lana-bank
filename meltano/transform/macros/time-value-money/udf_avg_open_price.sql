{% macro create_udf_avg_open_price() %}

CREATE OR REPLACE FUNCTION {{target.schema}}.udf_avg_open_price (
	deposit ARRAY<FLOAT64>, open_price ARRAY<FLOAT64>
) RETURNS ARRAY<FLOAT64> LANGUAGE js AS r"""
	var avg_open_price = [];
	var balance = 0.0;
	var current_avg_open_price = 0.0;
	for (var i = 0; i < deposit.length; i++) {
		balance = balance + deposit[i];
		if (deposit[i] > 0) {
			current_avg_open_price = ((balance-deposit[i])*current_avg_open_price + deposit[i]*open_price[i]) / balance;
		}
		avg_open_price.push(current_avg_open_price);
	}
	return avg_open_price;
"""

{% endmacro %}
