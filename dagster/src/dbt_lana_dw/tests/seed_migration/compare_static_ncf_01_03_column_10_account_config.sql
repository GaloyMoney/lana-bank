-- Compare legacy SQL model with new seed+wrapper implementation
-- This test should return 0 rows if both are equivalent
-- Arrays are compared by converting to JSON strings

(
    select order_by, title, eng_title,
           to_json_string(source_account_codes) as source_account_codes_json,
           column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_10_account_config_legacy') }}
    except distinct
    select order_by, title, eng_title,
           to_json_string(source_account_codes) as source_account_codes_json,
           column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_10_account_config') }}
)
union all
(
    select order_by, title, eng_title,
           to_json_string(source_account_codes) as source_account_codes_json,
           column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_10_account_config') }}
    except distinct
    select order_by, title, eng_title,
           to_json_string(source_account_codes) as source_account_codes_json,
           column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_10_account_config_legacy') }}
)
