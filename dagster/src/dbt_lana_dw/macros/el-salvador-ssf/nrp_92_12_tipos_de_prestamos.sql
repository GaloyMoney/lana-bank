{% macro nrp_92_12_tipos_de_prestamos(key) %}
{{ return({
	"Crédito decreciente": "CD",
	"Crédito rotativo": "CR",
	"Decreciente no rotativo": "NR",
	"Tarjeta de crédito": "TC",
	"Créditos interbancarios": "CI",
	"Descuento de letras o factoraje": "DI",
	"Créditos de tesorería": "CT",
	"Sobregiros autorizados": "SA",
	"Sobregiros ocasionales": "SO",
	"Créditos puente": "CU",
}.get(key)) }}
{% endmacro %}
