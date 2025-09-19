{% set min_usd_amount_to_report = 25000 %}

with seed_bank_address as (select * from {{ ref("seed_bank_address") }}),
int_core_withdrawal_events_rollup_sequence as (select * from {{ ref("int_core_withdrawal_events_rollup_sequence") }}),
int_core_withdrawal_events_rollup as (select * from {{ ref("int_core_withdrawal_events_rollup") }}),
int_core_deposit_events_rollup_sequence as (select * from {{ ref("int_core_deposit_events_rollup_sequence") }}),
int_core_deposit_events_rollup as (select * from {{ ref("int_core_deposit_events_rollup") }}),
int_core_deposit_account_events_rollup as (select * from {{ ref("int_core_deposit_account_events_rollup") }}),
stg_core_public_ids as (select * from {{ ref("stg_core_public_ids") }}),
relevant_withdrawals as (
    select withdrawal_id
    from int_core_withdrawal_events_rollup_sequence
    where is_confirmed = true
        and amount_usd >= {{ min_usd_amount_to_report }}
    group by withdrawal_id
),
withdrawal_confirmation_timestamps as (
    select 
        ers.withdrawal_id,
        min(ers.withdrawal_modified_at) as withdrawal_confirmed_at
    from int_core_withdrawal_events_rollup_sequence ers
    inner join relevant_withdrawals rw 
        on ers.withdrawal_id = rw.withdrawal_id
    where ers.is_confirmed = true
    group by ers.withdrawal_id
),
relevant_deposits as (
    select deposit_id
    from int_core_deposit_events_rollup_sequence
    where status = 'Confirmed'
        and amount_usd >= {{ min_usd_amount_to_report }}
    group by deposit_id
),
deposit_confirmation_timestamps as (
    select 
        ers.deposit_id,
        min(ers.deposit_modified_at) as deposit_confirmed_at
    from int_core_deposit_events_rollup_sequence ers
    inner join relevant_deposits rd
        on ers.deposit_id = rd.deposit_id
    where status = 'Confirmed'
    group by deposit_id
),
withdrawal_transactions as (
select
    withdrawal_public_ids.id as numeroRegistroBancario,
    JSON_OBJECT(
        'direccionAgencia', bank_address.full_address,
        'idDepartamento', bank_address.region_id,
        'idMunicipio', bank_address.town_id
    ) as estacionServicio,
    wct.withdrawal_confirmed_at as fechaTransaccion,
    CAST(null AS INTEGER) as tipoPersonaA,
    null as detallesPersonaA,
    CAST(null AS INTEGER) as tipoPersonaB,
    null as detallesPersonaB,
    aer.public_id as numeroCuentaPO,
    "Cuenta Corriente" as claseCuentaPO,
    wer.reference as conceptoTransaccionPO,
    wer.amount_usd as valorOtrosMediosElectronicosPO,
    CAST(null AS STRING) as numeroProductoPB,
    CAST(null AS STRING) as claseCuentaPB,
    wer.amount_usd as montoTransaccionPB,
    wer.amount_usd as valorMedioElectronicoPB,
    CAST(null AS STRING) as bancoCuentaDestinatariaPB
from int_core_withdrawal_events_rollup wer
inner join relevant_withdrawals rw 
    on wer.withdrawal_id = rw.withdrawal_id
left join withdrawal_confirmation_timestamps wct
    on wer.withdrawal_id = wct.withdrawal_id
left join int_core_deposit_account_events_rollup aer
    on wer.deposit_account_id = aer.deposit_account_id
left join stg_core_public_ids as withdrawal_public_ids
    on wer.withdrawal_id = withdrawal_public_ids.target_id
cross join -- Note: this assumes there's only one address!
seed_bank_address as bank_address
),
deposit_transactions as (
select
    deposit_public_ids.id as numeroRegistroBancario,
    JSON_OBJECT(
        'direccionAgencia', bank_address.full_address,
        'idDepartamento', bank_address.region_id,
        'idMunicipio', bank_address.town_id
    ) as estacionServicio,
    dct.deposit_confirmed_at as fechaTransaccion,
    CAST(null AS INTEGER) as tipoPersonaA,
    null as detallesPersonaA,
    CAST(null AS INTEGER) as tipoPersonaB,
    null as detallesPersonaB,
    CAST(null AS STRING) as numeroCuentaPO,
    CAST(null AS STRING) as claseCuentaPO,
    der.reference as conceptoTransaccionPO,
    der.amount_usd as valorOtrosMediosElectronicosPO,
    aer.public_id as numeroProductoPB,
    "Cuenta Corriente" as claseCuentaPB,
    der.amount_usd as montoTransaccionPB,
    der.amount_usd as valorMedioElectronicoPB,
    CAST(null AS STRING) as bancoCuentaDestinatariaPB
from int_core_deposit_events_rollup der
inner join relevant_deposits rd 
    on der.deposit_id = rd.deposit_id
left join deposit_confirmation_timestamps dct
    on der.deposit_id = dct.deposit_id
left join int_core_deposit_account_events_rollup aer
    on der.deposit_account_id = aer.deposit_account_id
left join stg_core_public_ids as deposit_public_ids
    on der.deposit_id = deposit_public_ids.target_id
cross join -- Note: this assumes there's only one address!
seed_bank_address as bank_address
)

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
from withdrawal_transactions
union all
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
from deposit_transactions
