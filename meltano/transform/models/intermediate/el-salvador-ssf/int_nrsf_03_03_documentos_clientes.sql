with

customers as (
    select *
    from {{ ref('int_core_customer_events_rollup') }}
    left join {{ ref('int_customer_identities') }} using (customer_id)

)
,

final as (

    select
        customer_id,
        'NIT' as `Código del Documento`,
        tax_id_number as `Número de documento`
    from customers
    where tax_id_number is not null

    union all

    select
        customer_id,
        'DUI' as `Código del Documento`,
        dui as `Número de documento`
    from customers
    where dui is not null

    union all

    select
        customer_id,
        'PASAP' as `Código del Documento`,
        passport_number as `Número de documento`
    from customers
    where passport_number is not null
)

select
    customer_public_ids.id as `NIU`,
    `Código del Documento`,
    `Número de documento`
from
    final
left join
    {{ ref('stg_core_public_ids') }} as customer_public_ids
    on customer_id = customer_public_ids.target_id
