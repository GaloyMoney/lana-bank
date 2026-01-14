select
    left(`Número de la cuenta`, 20) as `Número de la cuenta`,
    cast(`Monto de ajuste` as string) as `Monto de ajuste`,
    left(`Detalle del ajuste`, 100) as `Detalle del ajuste`
from {{ ref("int_nrsf_03_09_ajustes") }}
