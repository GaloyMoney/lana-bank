with seed_bank_address as (select * from {{ ref("seed_bank_address") }}),
int_core_withdrawal_events_rollup as (select * from {{ ref("int_core_withdrawal_events_rollup")}})
select
    null as numeroRegistroBancario,
    JSON_OBJECT(
        'direccionAgencia', bank_address.full_address,
        'idDepartamento', bank_address.region_id,
        'idMunicipio', bank_address.town_id
    ) as estacionServicio,
    null as fechaTransaccion,
    null as tipoPersonaA,
    null as detallesPersonaA,
    null as tipoPersonaB,
    null as detallesPersonaB,
    null as numeroCuentaPO,
    null as claseCuentaPO,
    null as conceptoTransaccionPO,
    null as valorOtrosMediosElectronicosPO,
    null as numeroProductoPB,
    null as claseCuentaPB,
    null as montoTransaccionPB,
    null as valorMedioElectronicoPB,
    null as bancoCuentaDestinatariaPB
from seed_bank_address as bank_address
-- Note: this table should be cross joined to actual transactions, not accessed like this.
-- Right now it's just here cause there are no more tables.
