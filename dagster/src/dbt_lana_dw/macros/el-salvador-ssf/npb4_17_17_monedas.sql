{% macro npb4_17_17_monedas(key) %}
{{ return({
	"Dólares": "1",
	"Otras monedas": "2",
}.get(key, "2")) }}
{% endmacro %}
