select title as `CONCEPTO`, balance as `SALDO`
from {{ ref("int_nrp_28_01_reserva_de_liquidez") }}
order by order_by
