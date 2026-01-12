-- Simple wrapper model that selects from the seed
select * from {{ ref("static_ncf_01_03_column_config_seed") }} order by column_order_by
