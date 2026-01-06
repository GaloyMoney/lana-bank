with

    column_01 as (select * from {{ ref("static_ncf_01_03_column_01_account_config") }}),

    column_02 as (select * from {{ ref("static_ncf_01_03_column_02_account_config") }}),

    column_03 as (select * from {{ ref("static_ncf_01_03_column_03_account_config") }}),

    column_04 as (select * from {{ ref("static_ncf_01_03_column_04_account_config") }}),

    column_05 as (select * from {{ ref("static_ncf_01_03_column_05_account_config") }}),

    column_06 as (select * from {{ ref("static_ncf_01_03_column_06_account_config") }}),

    column_07 as (select * from {{ ref("static_ncf_01_03_column_07_account_config") }}),

    column_08 as (select * from {{ ref("static_ncf_01_03_column_08_account_config") }}),

    column_09 as (select * from {{ ref("static_ncf_01_03_column_09_account_config") }}),

    column_10 as (select * from {{ ref("static_ncf_01_03_column_10_account_config") }}),

    final as (
        select *
        from column_01
        union all
        select *
        from column_02
        union all
        select *
        from column_03
        union all
        select *
        from column_04
        union all
        select *
        from column_05
        union all
        select *
        from column_06
        union all
        select *
        from column_07
        union all
        select *
        from column_08
        union all
        select *
        from column_09
        union all
        select *
        from column_10
    )

select *
from final
order by order_by
