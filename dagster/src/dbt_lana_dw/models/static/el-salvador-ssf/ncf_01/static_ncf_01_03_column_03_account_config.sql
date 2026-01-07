-- Wrapper model that joins row titles with column config (Other Reserves)
with
    titles as (
        select
            order_by,
            title,
            eng_title,
            array(
                select json_value(item, '$')
                from unnest(json_query_array(source_account_codes, '$')) as item
            ) as source_account_codes
        from {{ ref("static_ncf_01_03_row_titles_seed") }}
    ),

    column_cfg as (select * from {{ ref("static_ncf_01_03_column_config") }})

select
    titles.*,
    column_cfg.column_order_by,
    column_cfg.column_title,
    column_cfg.eng_column_title
from titles
left join column_cfg on column_cfg.eng_column_title = 'Other Reserves'
order by order_by
