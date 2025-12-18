{% macro npb4_17_09_tipos_de_garantias(key) %}
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
	"Valores de rescate de seguros de vida": "VR",
	"Póliza de seguro": "PO",
	"Prenda sobre valores de renta fija": "PV",
}.get(key)) }}
{% endmacro %}
