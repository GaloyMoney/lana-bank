{% macro npb4_17_07_estados_de_la_referencia(key) %}
{{ return({
	"Vigente": "1",
	"Vencido": "2",
	"Cancelado": "3",
	"Saneado": "4",
	"Vía Judicial": "5",
}.get(key)) }}
{% endmacro %}
