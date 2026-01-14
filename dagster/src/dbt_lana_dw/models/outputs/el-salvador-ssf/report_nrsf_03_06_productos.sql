select
    left(`Código del producto`, 4) as `Código del producto`,
    left(`Nombre del producto`, 30) as `Nombre del producto`,
    left(`Estado del producto`, 1) as `Estado del producto`,
    left(`Código genérico del producto`, 2) as `Código genérico del producto`
from
    {{ ref('int_nrsf_03_06_productos') }}
