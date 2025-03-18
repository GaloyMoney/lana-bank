{{ config(materialized='table') }}

select
    "tipo_deposito",
    "cod_banco",
    left("identificacion_garantia", 20) as "identificacion_garantia",
    left(replace("nit_depositante", '-', ''), 14) as "nit_depositante",
{% if target.type == 'bigquery' %}
    format_date('%Y-%m-%d', cast("fecha_deposito" as date)) as "fecha_deposito",
    format_date('%Y-%m-%d', cast("fecha_vencimiento" as date)) as "fecha_vencimiento",
    format('%.2f', round("valor_deposito", 2)) as "valor_deposito"
{% elif target.type == 'snowflake' %}
    TO_CHAR(cast("fecha_deposito" as date), 'YYYY-MM-DD') as "fecha_deposito",
    TO_CHAR(cast("fecha_vencimiento" as date), 'YYYY-MM-DD') as "fecha_vencimiento",
    TO_CHAR(round("valor_deposito", 2), '999999999.00') as "valor_deposito"
{% endif %}

from {{ ref('int_npb4_17_07_garantia_pignorada_xml_raw') }}
