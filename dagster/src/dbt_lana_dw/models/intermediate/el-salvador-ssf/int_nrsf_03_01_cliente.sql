with

    credit_facilities as (
        select
            customer_id,
            sum(total_collateral_amount_usd) as sum_total_collateral_amount_usd
        from {{ ref("int_approved_credit_facilities") }}
        group by customer_id
    ),

    customers as (
        select *
        from {{ ref("int_core_customer_events_rollup") }}
        left join {{ ref("int_customer_identities") }} using (customer_id)
        left join credit_facilities using (customer_id)
    )

select
    customer_public_ids.id as `NIU`,
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
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo("Deudores normales") }}'
    as `Clasificación de Riesgo`,
    relationship_to_bank as `Tipo de relación`,
    cast(null as string) as `Agencia`,
    least(
        sum_total_collateral_amount_usd, {{ var("deposits_coverage_limit") }}
    ) as `Saldo garantizado`
from customers
left join
    {{ ref("stg_core_public_ids") }} as customer_public_ids
    on customer_id = customer_public_ids.target_id
