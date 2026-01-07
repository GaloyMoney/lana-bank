-- Compare legacy SQL model with new seed implementation
-- This test should return 0 rows if both are equivalent

(
    select column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_config_legacy') }}
    except distinct
    select column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_config') }}
)
union all
(
    select column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_config') }}
    except distinct
    select column_order_by, column_title, eng_column_title
    from {{ ref('static_ncf_01_03_column_config_legacy') }}
)
