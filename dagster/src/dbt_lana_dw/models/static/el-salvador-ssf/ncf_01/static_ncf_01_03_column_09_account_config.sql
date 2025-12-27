with

titles as (

    select
        100 as order_by,
        'Balance al 01 de enero de {YEAR}' as title,
        'Balance as of January 1, {YEAR}' as eng_title,
        cast([] as array<string>) as source_account_codes
    union all
    select
        200,
        'Efecto de las correcciones de errores',
        'Effect of bug fixes',
        []
    union all
    select
        300,
        'Efecto de los cambios en políticas contables',
        'Effect of changes in accounting policies',
        []
    union all
    select
        400,
        'Balance re-expresado',
        'Restated balance sheet',
        []
    union all
    select
        500,
        'Emisión de acciones',
        'Issuance of shares',
        []
    union all
    select
        600,
        'Dividendos',
        'Dividends',
        []
    union all
    select
        700,
        'Otro Resultado Integral:',
        'Other Comprehensive Income:',
        []
    union all
    select
        800,
        'Incrementos en elementos que no se reclasificaran en resultados',
        'Increases in items that will not be reclassified in results',
        []
    union all
    select
        900,
        'Disminuciones en elementos que no se reclasificaran en resultados',
        'Decreases in items that will not be reclassified in results',
        []
    union all
    select
        1000,
        'Incremento en elementos que se reclasificaran en resultados',
        'Increase in items to be reclassified in results',
        []
    union all
    select
        1100,
        'Reclasificaciones a resultados',
        'Reclassifications to results',
        []
    union all
    select
        1200,
        'Otros aumentos o (-) disminuciones del patrimonio neto',
        'Other increases or (-) decreases in net worth',
        []
    union all
    select
        1300,
        'Balance al 31 de diciembre de {YEAR}',
        'Balance sheet as of December 31, {YEAR}',
        []

),

column as (
    select *
    from {{ ref('static_ncf_01_03_column_config') }}
)

select *
from titles
left join column on eng_column_title = "Non-controlling shareholders' shares"
order by order_by
