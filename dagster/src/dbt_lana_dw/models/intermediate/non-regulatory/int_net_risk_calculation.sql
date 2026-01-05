with

final as (
    select
        credit_facility_id as line_of_credit,
        disbursal_id as disbursement_number,
        remaining_balance_usd as principal_balance,
        0 as interest,
        `saldo_referencia` as total_debt,
        `valor_garantia_cons` as guarantee_amount,
        100 as percent_by_risk_category,
        risk_category_ref as category_b,
        net_risk,
        reserve_percentage,
        reserve
    from {{ ref('int_nrp_41_02_referencia') }}
)

select * from final
