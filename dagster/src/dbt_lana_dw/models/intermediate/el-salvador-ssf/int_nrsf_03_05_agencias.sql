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
    'TODO' as `Código de la Agencia`,
    'TODO' as `Nombre de la Agencia`,
    'TODO' as `Ubicación de la Agencia`,
    'TODO' as `Código del Departamento`,
    'TODO' as `Código del Distrito`,
    'TODO' as `Estado de la Agencia`
from
    final
