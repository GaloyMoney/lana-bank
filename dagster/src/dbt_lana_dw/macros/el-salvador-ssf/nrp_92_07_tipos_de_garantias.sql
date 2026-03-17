{% macro nrp_92_07_tipos_de_garantias(key) %}
{{ return({
	"Hipoteca abierta": "HA",
	"Hipoteca cerrada": "HC",
	"Fiduciaria": "FI",
	"Prendaria": "PR",
	"Pignorada - Depósito de dinero": "PI",
	"Fondos de garantías": "FG",
	"Fianzas de bancos locales o bancos extranjeros de primera línea": "FB",
	"Cartas de crédito stand by": "CC",
	"Avales": "AV",
	"Bonos de prenda": "BP",
	"Prenda de documentos": "PD",
	"Prenda sobre valores de renta fija": "PV",
	"Prenda sobre Criptomonedas": "PC",
	"Prenda sobre Stablecoins": "ST",
	"Prenda Activos de Fácil Liquidación": "AF",
}.get(key)) }}
{% endmacro %}
