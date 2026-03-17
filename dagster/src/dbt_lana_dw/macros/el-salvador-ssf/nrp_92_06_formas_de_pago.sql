{% macro nrp_92_06_formas_de_pago(key) %}
{{ return({
	"Anual": "A",
	"Semestral": "E",
	"Trimestral": "T",
	"Bimensual": "B",
	"Mensual": "M",
	"Quincenal": "Q",
	"Semanal": "S",
	"Diario": "D",
	"Al Vencimiento": "V",
	"Pactada": "P",
	"Otras": "O",
}.get(key)) }}
{% endmacro %}
