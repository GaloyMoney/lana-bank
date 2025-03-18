{{ config(materialized='table') }}

select
    left("num_referencia", 20) as "num_referencia",
    left("cod_cartera", 2) as cod_cartera,
    left("cod_activo", 2) as cod_activo,
    left("identificacion_garantia", 20) as identificacion_garantia,
    left("tipo_garantia", 2) as tipo_garantia,
{% if target.type == 'bigquery' %}
    format('%.2f', round("valor_garantia_proporcional", 2))
        as valor_garantia_proporcional
{% elif target.type == 'snowflake' %}
    TO_CHAR(round("valor_garantia_proporcional", 2), '999999999.00')
        as valor_garantia_proporcional
{% endif %}

from {{ ref('int_npb4_17_03_referencia_garantia_xml_raw') }}
