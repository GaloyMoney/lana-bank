with dummy as (

    select
        null as `cod_cartera`,
        null as `cod_activo`,
        null as `num_referencia`,
        null as `codigo_gasto`,
        null as `tipo_gasto`,
        null as `monto_gasto`

)

select *
from dummy
where false
