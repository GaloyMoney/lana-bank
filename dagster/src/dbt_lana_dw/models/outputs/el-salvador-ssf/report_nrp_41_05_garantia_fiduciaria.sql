with dummy as (

    select
        null as `num_referencia`,
        null as `cod_cartera`,
        null as `cod_activo`,
        null as `nit_fiador_codeudor`,
        null as `fiador_codeudor`

)

select *
from dummy
where false
