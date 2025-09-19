with

customers as (
    select *
    from {{ ref('int_core_customer_events_rollup') }}
    left join {{ ref('int_customer_identities') }} using (customer_id)
    where customer_type = 'BankEmployee' and 1 = 0
)

select
    married_name as `Apellido de casada`,
    '2008-10-31' as `Fecha de ingreso`,
    'TODO' as `Cargo`,
    'TODO' as `Código del documento`,
    'TODO' as `Número de documento`,
    'TODO' as `Número Telefónico`,
    'TODO' as `Departamento`,
    '0' as `Relacionado por administración`,
    split(first_name, ' ')[safe_offset(0)] as `Primer Nombre`,
    split(first_name, ' ')[safe_offset(1)] as `Segundo Nombre`,
    split(last_name, ' ')[safe_offset(0)] as `Primer Apellido`,
    split(last_name, ' ')[safe_offset(1)] as `Segundo Apellido`,
    left(replace(customer_id, '-', ''), 14) as `NIU`
from
    customers
