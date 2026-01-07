-- Compare legacy SQL model with new seed+wrapper implementation
-- This test should return 0 rows if both are equivalent
-- Arrays are compared by converting to JSON strings

(
    select order_by, title, eng_title,
           to_json_string(sum_account_codes) as sum_account_codes_json,
           to_json_string(diff_account_codes) as diff_account_codes_json
    from {{ ref('static_nrp_28_01_account_config_legacy') }}
    except distinct
    select order_by, title, eng_title,
           to_json_string(sum_account_codes) as sum_account_codes_json,
           to_json_string(diff_account_codes) as diff_account_codes_json
    from {{ ref('static_nrp_28_01_account_config') }}
)
union all
(
    select order_by, title, eng_title,
           to_json_string(sum_account_codes) as sum_account_codes_json,
           to_json_string(diff_account_codes) as diff_account_codes_json
    from {{ ref('static_nrp_28_01_account_config') }}
    except distinct
    select order_by, title, eng_title,
           to_json_string(sum_account_codes) as sum_account_codes_json,
           to_json_string(diff_account_codes) as diff_account_codes_json
    from {{ ref('static_nrp_28_01_account_config_legacy') }}
)
