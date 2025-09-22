{{ config(materialized='table') }}

select
    cast(round(`enero`, 2) as string) as `enero`,
    cast(round(`febrero`, 2) as string) as `febrero`,
    cast(round(`marzo`, 2) as string) as `marzo`,
    cast(round(`abril`, 2) as string) as `abril`,
    cast(round(`mayo`, 2) as string) as `mayo`,
    cast(round(`junio`, 2) as string) as `junio`,
    cast(round(`julio`, 2) as string) as `julio`,
    cast(round(`agosto`, 2) as string) as `agosto`,
    cast(round(`septiembre`, 2) as string) as `septiembre`,
    cast(round(`octubre`, 2) as string) as `octubre`,
    cast(round(`noviembre`, 2) as string) as `noviembre`,
    cast(round(`diciembre`, 2) as string) as `diciembre`,
    left(`id_codigo_cuentaproy`, 10) as `id_codigo_cuentaproy`,
    left(`nom_cuentaproy`, 80) as `nom_cuentaproy`
from
    {{ ref('int_nrp_51_08_balance_proyectado') }}
