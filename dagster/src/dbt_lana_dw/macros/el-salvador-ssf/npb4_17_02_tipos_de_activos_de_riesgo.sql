{% macro npb4_17_02_tipos_de_activos_de_riesgo(key) %}
{{ return({
	"Préstamos": "PD",
	"Cuentas por cobrar": "CP",
	"Contingentes cartas de crédito": "CC",
	"Contingentes fianzas/avales/garantías": "FA",
	"Primas documentadas": "PR",
	"Referencias saneadas": "SN",
}.get(key)) }}
{% endmacro %}
