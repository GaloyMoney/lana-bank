with seed_bank_address as (select * from {{ ref("seed_bank_address") }}),
int_core_withdrawal_events_rollup_sequence as (select * from {{ ref("int_core_withdrawal_events_rollup_sequence") }}) ,
int_core_withdrawal_events_rollup as (select * from {{ ref("int_core_withdrawal_events_rollup") }}),
int_core_deposit_account_events_rollup as (select * from {{ ref("int_core_deposit_account_events_rollup") }}),
approved_withdrawals as (
    select withdrawal_id
    from int_core_withdrawal_events_rollup_sequence
    where is_confirmed = true
    group by withdrawal_id
),
withdrawal_confirmation_timestamps as (
    select 
        ers.withdrawal_id,
        min(withdrawal_modified_at) as withdrawal_confirmed_at
    from int_core_withdrawal_events_rollup_sequence ers
    inner join approved_withdrawals aw 
        on ers.withdrawal_id = aw.withdrawal_id
    where ers.is_confirmed = true
    group by ers.withdrawal_id
)
select
    wer.withdrawal_id as numeroRegistroBancario, -- probably this should be a public id and not the private uuid
    JSON_OBJECT(
        'direccionAgencia', bank_address.full_address,
        'idDepartamento', bank_address.region_id,
        'idMunicipio', bank_address.town_id
    ) as estacionServicio,
    wct.withdrawal_confirmed_at as fechaTransaccion,
    null as tipoPersonaA,
    null as detallesPersonaA,
    null as tipoPersonaB,
    null as detallesPersonaB,
    aer.public_id as numeroCuentaPO,
    "Cuenta Corriente" as claseCuentaPO,
    null as conceptoTransaccionPO,
    wer.amount_usd as valorOtrosMediosElectronicosPO,
    null as numeroProductoPB,
    null as claseCuentaPB,
    wer.amount_usd as montoTransaccionPB,
    wer.amount_usd as valorMedioElectronicoPB,
    null as bancoCuentaDestinatariaPB
from int_core_withdrawal_events_rollup wer
inner join approved_withdrawals aw 
    on wer.withdrawal_id = aw.withdrawal_id
left join withdrawal_confirmation_timestamps wct
    on aw.withdrawal_id = wct.withdrawal_id
left join int_core_deposit_account_events_rollup aer
    on wer.deposit_account_id = aer.deposit_account_id
cross join -- Note: this assumes there's only one address!
seed_bank_address as bank_address
