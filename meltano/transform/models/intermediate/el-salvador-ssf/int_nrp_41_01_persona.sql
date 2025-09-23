--TODO: business onboarding

select

    -- use NIU type (`tipo_identificador` = 'N')
    customer_public_ids.id as `nit_persona`,

    dui,

    upper(split(last_name, ' ')[safe_offset(0)]) as `primer_apellido`,
    upper(split(last_name, ' ')[safe_offset(1)]) as `segundo_apellido`,
    upper(split(first_name, ' ')[safe_offset(0)]) as `primer_nombre`,
    upper(split(first_name, ' ')[safe_offset(1)]) as `segundo_nombre`,
    cast(null as string) as `apellido_casada`,

    -- NULL for natural person
    cast(null as string) as `nombre_sociedad`,

    -- '1' for natural person
    '1' as `tipo_persona`,

    relationship_to_bank as `tipo_relacion`,

    -- 'U' for non-Salvadoran using the most flexible Unique Identification Number
    'U' as `tipo_identificador`,

    -- NULL for non-Salvadoran
    cast(null as string) as `nit_desactualizado`,

    case
        when country_of_residence_alpha_3_code = 'SLV' then 'Y'
        else 'N'
    end as `residente`,

    -- codified main economic activity of the person,
    -- i.e. the one that generates the greatest cash flow
    economic_activity_code as `giro_persona`,

    cast(null as string) as `tamano_empresa`,
    cast(null as string) as `tipo_empresa`,

    -- Provision of sanitation reserves established accounted for by the entity for each debtor
    -- Since the value of the collateral is always greater than the value of the loan:
    0.0 as `reserva`,

    -- codified risk category assigned to the debtor depending of the status of the loan
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo('Deudores normales') }}' as `categoria_riesgo`,

    customer_public_ids.id as `numero_cliente`,

    -- passport number / social security number / driver's license number / id card number
    passport_number as `id_alterno`,

    -- 'PS' for passport / 'SS' for social security / 'LI' for driver's license / 'CI' for id card
    'PS' as `tipo_id_alterno`,

    date_of_birth as `fecha_nacimiento`,
    country_of_residence_code as `pais_residencia`,

    -- Sum of the balances of the references that the person has plus the accrued interest
    -- Since the value of the collateral is always greater than the value of the loan:
    0.0 as `riesgo_consolidado`,

    gender as `sexo_persona`,
    occupation_code as `ocupación`,

    -- TIN (Tax Identification Number) issued by the country of origin
    tax_id_number as `id_pais_origen`,

    nationality_code as `nacionalidad`,
    cast(null as string) as `nit_anterior`,
    cast(null as string) as `tipo_ident_anterior`,

    el_salvador_municipality as `distrito_residencia`

from {{ ref('int_core_customer_events_rollup') }}
inner join {{ ref('int_customer_identities') }} using (customer_id)
left join
    {{ ref('stg_core_public_ids') }} as customer_public_ids
    on customer_id = customer_public_ids.target_id
