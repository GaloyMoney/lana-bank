with
    dummy as (

        select
            null as `identificacion_garantia`,
            null as `documento_depositante`,
            null as `fecha_deposito`,
            null as `fecha_vencimiento`,
            null as `valor_deposito`,
            null as `tipo_deposito`,
            null as `cod_banco`

    )

select *
from dummy
where false
