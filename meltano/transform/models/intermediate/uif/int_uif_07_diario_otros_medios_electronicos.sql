with seed_bank_address as (select * from {{ ref("seed_bank_address") }}),
int_core_withdrawal_events_rollup_sequence as (select * from {{ ref("int_core_withdrawal_events_rollup_sequence") }}),
int_core_withdrawal_events_rollup as (select * from {{ ref("int_core_withdrawal_events_rollup") }}),
int_core_deposit_events_rollup_sequence as (select * from {{ ref("int_core_deposit_events_rollup_sequence") }}),
int_core_deposit_events_rollup as (select * from {{ ref("int_core_deposit_events_rollup") }}),
int_core_deposit_account_events_rollup as (select * from {{ ref("int_core_deposit_account_events_rollup") }}),
confirmed_withdrawals as (
    select withdrawal_id
    from int_core_withdrawal_events_rollup_sequence
    where is_confirmed = true
    group by withdrawal_id
),
withdrawal_confirmation_timestamps as (
    select 
        ers.withdrawal_id,
        min(ers.withdrawal_modified_at) as withdrawal_confirmed_at
    from int_core_withdrawal_events_rollup_sequence ers
    inner join confirmed_withdrawals cw 
        on ers.withdrawal_id = cw.withdrawal_id
    where ers.is_confirmed = true
    group by ers.withdrawal_id
),
confirmed_deposits as (
    select deposit_id
    from int_core_deposit_events_rollup_sequence
    where status = 'Confirmed'
    group by deposit_id
),
deposit_confirmation_timestamps as (
    select 
        ers.deposit_id,
        min(ers.deposit_modified_at) as deposit_confirmed_at
    from int_core_deposit_events_rollup_sequence ers
    inner join confirmed_deposits cd
        on ers.deposit_id = cd.deposit_id
    where status = 'Confirmed'
    group by deposit_id
),
withdrawal_transactions as (
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
inner join confirmed_withdrawals cw 
    on wer.withdrawal_id = cw.withdrawal_id
left join withdrawal_confirmation_timestamps wct
    on wer.withdrawal_id = wct.withdrawal_id
left join int_core_deposit_account_events_rollup aer
    on wer.deposit_account_id = aer.deposit_account_id
cross join -- Note: this assumes there's only one address!
seed_bank_address as bank_address
),
deposit_transactions as (
select
    der.deposit_id as numeroRegistroBancario, -- probably this should be a public id and not the private uuid
    JSON_OBJECT(
        'direccionAgencia', bank_address.full_address,
        'idDepartamento', bank_address.region_id,
        'idMunicipio', bank_address.town_id
    ) as estacionServicio,
    dct.deposit_confirmed_at as fechaTransaccion,
    null as tipoPersonaA,
    null as detallesPersonaA,
    null as tipoPersonaB,
    null as detallesPersonaB,
    null as numeroCuentaPO,
    null as claseCuentaPO,
    null as conceptoTransaccionPO,
    der.amount_usd as valorOtrosMediosElectronicosPO,
    aer.public_id as numeroProductoPB,
    "Cuenta Corriente" as claseCuentaPB,
    der.amount_usd as montoTransaccionPB,
    der.amount_usd as valorMedioElectronicoPB,
    null as bancoCuentaDestinatariaPB
from int_core_deposit_events_rollup der
inner join confirmed_deposits cd 
    on der.deposit_id = cd.deposit_id
left join deposit_confirmation_timestamps dct
    on der.deposit_id = dct.deposit_id
left join int_core_deposit_account_events_rollup aer
    on der.deposit_account_id = aer.deposit_account_id
cross join -- Note: this assumes there's only one address!
seed_bank_address as bank_address
)
