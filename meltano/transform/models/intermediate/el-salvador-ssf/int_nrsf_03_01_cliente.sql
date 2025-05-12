with

credit_facilities as (
    select
        customer_id,
        sum(total_collateral) as sum_total_collateral
    from {{ ref('int_approved_credit_facilities') }}
    group by customer_id
),

customers as (
    select *
    from {{ ref('int_customers') }}
    left join {{ ref('int_customer_identities') }} using (customer_id)
    left join credit_facilities using (customer_id)
)

select
    left(replace(customer_id, '-', ''), 14) as `NIU`,
    split(first_name, ' ')[safe_offset(0)] as `Primer Nombre`,
    split(first_name, ' ')[safe_offset(1)] as `Segundo Nombre`,
    cast(null as string) as `Tercer Nombre`,
    split(last_name, ' ')[safe_offset(0)] as `Primer Apellido`,
    split(last_name, ' ')[safe_offset(1)] as `Segundo Apellido`,
    married_name as `Apellido de casada`,
    cast(null as string) as `Razón social`,
    '1' as `Tipo de persona`,
    cast(nationality_code as string) as `Nacionalidad`,
    cast(economic_activity_code as string) as `Actividad Económica`,
    cast(country_of_residence_code as string) as `País de Residencia`,
    '15' as `Departamento`,
    '00' as `Distrito`,
    formatted_address as `Dirección`,
    phone_number as `Número de teléfono fijo`,
    phone_number as `Número de celular`,
    email as `Correo electrónico`,
    '0' as `Es residente`,
    '1' as `Tipo de sector`,
    date_of_birth as `Fecha de Nacimiento`,
    gender as `Género`,
    marital_status as `Estado civil`,
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo('Deudores normales') }}'
        as `Clasificación de Riesgo`,
    relationship_to_bank as `Tipo de relación`,
    cast(null as string) as `Agencia`,
    least(sum_total_collateral, {{ var('deposits_coverage_limit') }}) as `Saldo garantizado`
from
    customers
