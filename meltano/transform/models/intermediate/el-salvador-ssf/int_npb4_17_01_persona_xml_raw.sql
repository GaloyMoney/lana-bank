select

    -- use NIU type ("tipo_identificador" = 'N')
    left(replace(customer_id, '-', ''), 14) as "nit_persona",

    -- NULL for non-Salvadoran ("nacionalidad" != '9300')
    cast(null as {{ dbt.type_string() }}) as "dui",

    upper(first_name) as "primer_apellido",
    cast(null as {{ dbt.type_string() }}) as "segundo_apellido",
    cast(null as {{ dbt.type_string() }}) as "apellido_casada",
    upper(last_name) as "primer_nombre",
    cast(null as {{ dbt.type_string() }}) as "segundo_nombre",

    -- NULL for natural person
    cast(null as {{ dbt.type_string() }}) as "nombre_sociedad",

    -- '1' for natural person
    '1' as "tipo_persona",

    -- '0' for natural person
    '0' as "tipo_relacion",

    -- 'U' for non-Salvadoran using the most flexible Unique Identification Number
    'U' as "tipo_identificador",

    -- NULL for non-Salvadoran
    cast(null as {{ dbt.type_string() }}) as "nit_desactualizado",

    -- 'N' for non-Salvadoran
    'N' as "residente",

    -- codified main economic activity of the person,
    -- i.e. the one that generates the greatest cash flow
    economic_activity_code as "giro_persona",

    cast(null as {{ dbt.type_string() }}) as "tamano_empresa",
    cast(null as {{ dbt.type_string() }}) as "tipo_empresa",

    -- Provision of sanitation reserves established accounted for by the entity for each debtor
    7060.0 as "reserva",

    -- codified risk category assigned to the debtor depending of the status of the loan
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo('Deudores normales') }}' as "categoria_riesgo",

    right(replace(customer_id, '-', ''), 17) as "numero_cliente",

    -- passport number / social security number / driver's license number / id card number
    passport_number as "id_alterno",

    -- 'PS' for passport / 'SS' for social security / 'LI' for driver's license / 'CI' for id card
    'PS' as "tipo_id_alterno",

    date_of_birth as "fecha_nacimiento",
    country_of_residence_code as "pais_residencia",

    -- Sum of the balances of the references that the person has plus the accrued interest
    -- TODO: use real number
    7060.0 as "riesgo_consolidado",

    gender as "sexo_persona",
    occupation_code as "ocupación",

    -- TIN (Tax Identification Number) issued by the country of origin
    tax_id_number as "id_pais_origen",


    nationality_code as "nacionalidad",
    cast(null as {{ dbt.type_string() }}) as "nit_anterior",
    cast(null as {{ dbt.type_string() }}) as "tipo_ident_anterior",
    cast(null as {{ dbt.type_string() }}) as "municipio_residencia"

from {{ ref('int_customers') }}
left join {{ ref('int_customer_identities') }} using (customer_id)
