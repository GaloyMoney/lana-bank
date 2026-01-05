{% macro npb4_17_08_formas_de_pago(key) %}
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
}.get(key, "O")) }}
{% endmacro %}
