{{ config(materialized='table') }}

select
    right(id_codigo_cuenta, 10) as id_codigo_cuenta,

{% if target.type == 'bigquery' %}
    -- FIXME
    left(regexp_replace(nom_cuenta, r'[&<>"]', '_'), 80) as nom_cuenta,
    format('%.2f', round(valor, 2)) as valor
{% elif target.type == 'snowflake' %}
    -- FIXME
    left(nom_cuenta, 80) as nom_cuenta,
    TO_CHAR(round(valor, 2), '999999999.00') as valor
{% endif %}


from {{ ref('int_npb4_16_01_saldo_cuenta_xml_raw') }}
