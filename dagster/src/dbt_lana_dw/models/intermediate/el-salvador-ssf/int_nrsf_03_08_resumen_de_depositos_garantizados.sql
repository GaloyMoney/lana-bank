with

customers as (
    select *
    from {{ ref('int_core_customer_events_rollup') }}
    left join {{ ref('int_customer_identities') }} using (customer_id)
    where customer_type = 'NoType' and 1 = 0
    -- customer_type in (
    --  'Individual',
    --  'GovernmentEntity',
    --  'PrivateCompany',
    --  'Bank',
    --  'FinancialInstitution',
    --  'ForeignAgencyOrSubsidiary',
    --  'NonDomiciledCompany',
    -- )

)
,

final as (

    select *
    from customers
)

select
    7060 as `Correlativo`,
    cast(null as string) as `Tercer Nombre`,
    married_name as `Apellido de casada`,
    cast(null as string) as `Razón social`,
    'DUI' as `Código del Documento`,
    dui as `Número de documento`,
    7060.0 as `Total de cuentas`,
    7060 as `Saldo de capital`,
    0.0 as `Saldo de intereses`,
    7060 as `Saldo garantizado`,
    left(replace(customer_id, '-', ''), 14) as `NIU`,
    split(first_name, ' ')[safe_offset(0)] as `Primer Nombre`,
    split(first_name, ' ')[safe_offset(1)] as `Segundo Nombre`,
    split(last_name, ' ')[safe_offset(0)] as `Primer Apellido`,
    split(last_name, ' ')[safe_offset(1)] as `Segundo Apellido`
from
    final
