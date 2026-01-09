with
    int_uif_07_diario_otros_medios_electronicos as (
        select * from {{ ref("int_uif_07_diario_otros_medios_electronicos") }}
    )
select
    numeroregistrobancario,
    estacionservicio,
    fechatransaccion,
    tipopersonaa,
    detallespersonaa,
    tipopersonab,
    detallespersonab,
    numerocuentapo,
    clasecuentapo,
    conceptotransaccionpo,
    valorotrosmedioselectronicospo,
    numeroproductopb,
    clasecuentapb,
    montotransaccionpb,
    valormedioelectronicopb,
    bancocuentadestinatariapb
from int_uif_07_diario_otros_medios_electronicos
