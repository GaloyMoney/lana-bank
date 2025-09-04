with

reserve as (
    select *
    from {{ ref('int_nrp_28_01_reserva_de_liquidez_explain') }}
)

select
    title,
    balance,
from reserve
