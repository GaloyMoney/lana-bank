{{
    config(
        materialized="table",
        partition_by={"field": "as_of_date", "data_type": "date"},
    )
}}

select
    as_of_date,
    cast(format("%.2f", round(`valor`, 2)) as string) as `valor`,
    right(`id_codigo_cuenta`, 10) as `id_codigo_cuenta`,
    upper(left(regexp_replace(`nom_cuenta`, r'[&<>"]', "_"), 80)) as `nom_cuenta`
from {{ ref("int_nrp_51_01_saldo_cuenta_daily") }}
