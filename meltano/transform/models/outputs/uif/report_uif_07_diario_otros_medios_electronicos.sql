with int_uif_07_diario_otros_medios_electronicos as (select * from {{ ref("int_uif_07_diario_otros_medios_electronicos") }})
select
    numeroRegistroBancario,
    estacionServicio,
    fechaTransaccion,
    tipoPersonaA,
    detallesPersonaA,
    tipoPersonaB,
    detallesPersonaB,
    numeroCuentaPO,
    claseCuentaPO,
    conceptoTransaccionPO,
    valorOtrosMediosElectronicosPO,
    numeroProductoPB,
    claseCuentaPB,
    montoTransaccionPB,
    valorMedioElectronicoPB,
    bancoCuentaDestinatariaPB
from int_uif_07_diario_otros_medios_electronicos
