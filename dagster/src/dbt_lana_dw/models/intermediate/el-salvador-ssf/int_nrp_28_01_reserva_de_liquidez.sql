with

    reserve as (
        select order_by, title, sum(balance) as balance
        from {{ ref("int_nrp_28_01_reserva_de_liquidez_explain") }}
        group by order_by, title
    )

select order_by, title, balance
from reserve
