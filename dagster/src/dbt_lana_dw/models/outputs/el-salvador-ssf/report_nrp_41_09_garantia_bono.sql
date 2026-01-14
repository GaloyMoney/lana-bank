with dummy as (

    select
        null as `identificacion_garantia`,
        null as `tipo_prenda`,
        null as `descripción`,
        null as `fecha_certificado`,
        null as `valor_prenda`,
        null as `saldo_prenda`,
        null as `cod_almacenadora`

)

select *
from dummy
where false
