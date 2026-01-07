-- Compare legacy SQL model with new seed implementation
-- This test should return 0 rows if both are equivalent

(
    select account_code, account_name, eng_account_name, coefficient
    from {{ ref('static_nrp_28_01_liquidity_coefficients_legacy') }}
    except distinct
    select account_code, account_name, eng_account_name, coefficient
    from {{ ref('static_nrp_28_01_liquidity_coefficients') }}
)
union all
(
    select account_code, account_name, eng_account_name, coefficient
    from {{ ref('static_nrp_28_01_liquidity_coefficients') }}
    except distinct
    select account_code, account_name, eng_account_name, coefficient
    from {{ ref('static_nrp_28_01_liquidity_coefficients_legacy') }}
)
