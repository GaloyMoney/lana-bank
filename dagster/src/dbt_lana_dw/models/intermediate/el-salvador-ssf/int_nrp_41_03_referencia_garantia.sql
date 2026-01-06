with
    loans_and_credit_facilities as (
        select
            collateral_amount_usd,
            disbursal_id as reference_id,
            total_disbursed_usd as loan_amount_usd

        from {{ ref("int_approved_credit_facility_loans") }}

        where not matured

    )

select
    disbursement_public_ids.id as `num_referencia`,
    '{{ npb4_17_01_tipos_de_cartera("Cartera propia Ley Acceso al Crédito (19)") }}'
    as `cod_cartera`,
    '{{ npb4_17_02_tipos_de_activos_de_riesgo("Préstamos") }}' as `cod_activo`,
    disbursement_public_ids.id as `identificacion_garantia`,
    '{{ npb4_17_09_tipos_de_garantias("Pignorada - Depósito de dinero") }}'
    as `tipo_garantia`,
    coalesce(
        safe_divide(collateral_amount_usd, loan_amount_usd) * 100, 1
    ) as `valor_garantia_proporcional`

from loans_and_credit_facilities
left join
    {{ ref("stg_core_public_ids") }} as disbursement_public_ids
    on reference_id = disbursement_public_ids.target_id
