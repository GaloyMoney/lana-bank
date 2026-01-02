{% macro npb4_17_21_fuentes_de_recursos(key) %}
{{ return({
	"Recursos propios de la entidad": "01",
	"Recursos de bancos extranjeros de corto plazo": "02",
	"Recursos de bancos extranjeros de mediano y largo plazo": "03",
	"Recursos del Banco Central de Reserva de El Salvador (BCR)": "04",
	"Recursos del Banco de Desarrollo de la Republica de El Salvador (BANDESAL) (21)": "05",
	"Recursos del Banco Centroamericano de Integración Económica (BCIE)": "06",
	"Recursos del Banco Interamericano de Desarrollo (BID)": "07",
	"Recursos del Banco Mundial": "08",
	"Recursos del Fondo Nacional para la Vivienda Popular ( FONAVIPO)": "09",
	"Recursos del Fondo Social para la Vivienda (FSV)": "10",
	"Recursos de Certificados de Depósitos a Plazo de Vivienda (CEDEVIV)": "11",
	"Recursos de Certificados de Depósitos a Plazo Agropecuario (CEDEAGRO)": "12",
	"Recursos Préstamos BCR-BID": "13",
	"Recursos de otras fuentes": "99",
}.get(key, "99")) }}
{% endmacro %}
