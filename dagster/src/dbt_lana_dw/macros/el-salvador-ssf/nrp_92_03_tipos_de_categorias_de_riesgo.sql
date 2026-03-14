{% macro nrp_92_03_tipos_de_categorias_de_riesgo(key) %}
{{ return({
	"Deudores normales": "A1",
	"Deudores normales declinantes": "A2",
	"Deudores subnormales": "B",
	"Deudores deficientes": "C1",
	"Deudores deficientes declinantes": "C2",
	"Deudores de difícil recuperación": "D1",
	"Deudores de difícil recuperación declinantes": "D2",
	"Deudores irrecuperables": "E",
}.get(key)) }}
{% endmacro %}
