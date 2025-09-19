with

titles as (

    select
        100 as order_by,
        'ACTIVO' as title,
        'ASSET' as eng_title,
        [] as source_account_codes
    union all
    select
        200,
        'Efectivo y equivalentes de efectivo (111, 112 y todas aquellas partidas que la entidad conforme a sus políticas defina como equivalentes de efectivo)',
        'Cash and cash equivalents (111, 112 and all items that the entity defines as cash equivalents in accordance with its policies)',
        ['111', '112']
    union all
    select
        300,
        'Instrumentos financieros de inversión (neto) (1130, 1131, 1134)',
        'Investment financial instruments (net) (1130, 1131, 1134)',
        ['1130', '1131', '1134']
    union all
    select
        400,
        '    A Valor razonable con cambios en resultados',
        '    At fair value through profit or loss',
        []
    union all
    select
        500,
        '    A Valor razonable con cambios en otro resultado integral (VRORI)',
        '    At fair value through other comprehensive income (FVII)',
        []
    union all
    select
        600,
        '    A Costo amortizado',
        '    At amortized cost',
        []
    union all
    select
        700,
        'Derivados financieros para coberturas (1132)',
        'Financial derivatives for hedging (1132)',
        ['1132']
    union all
    select
        800,
        'Instrumentos Financieros Restringidos (1138)',
        'Restricted Financial Instruments (1138)',
        ['1138']
    union all
    select
        900,
        'Cartera de créditos (neta) (114)',
        'Loan portfolio (net) (114)',
        ['114']
    union all
    select
        1000,
        '    Créditos vigentes a un año plazo',
        '    Credits valid for one year',
        []
    union all
    select
        1100,
        '    Créditos vigentes a más de un año plazo',
        '    Credits valid for more than one year',
        []
    union all
    select
        1200,
        '    Créditos vencidos',
        '    Overdue credits',
        []
    union all
    select
        1300,
        '    (Estimación de pérdida por deterioro)',
        '    ( Estimated impairment loss)',
        []
    union all
    select
        1400,
        'Cuentas por cobrar (neto) (125)',
        'Accounts receivable (net) (125)',
        ['125']
    union all
    select
        1500,
        'Activos físicos e intangibles (neto) (13)',
        'Physical and intangible assets (net) (13)',
        ['13']
    union all
    select
        1600,
        'Activos extraordinarios (neto) (122)',
        'Extraordinary assets (net) (122)',
        ['122']
    union all
    select
        1700,
        'Activos de largo plazo mantenidos para la venta (127)',
        'Long-term assets held for sale (127)',
        ['127']
    union all
    select
        1800,
        'Inversiones en acciones (Neto) (126)',
        'Investments in shares (Net) (126)',
        ['126']
    union all
    select
        1900,
        'Otros Activos (121, 123, 124) (1)',
        'Other Assets (121, 123, 124) (1)',
        ['121', '123', '121']
    union all
    select
        2000,
        'Total Activos',
        'Total Assets',
        []
    union all
    select
        2100,
        'PASIVO',
        'PASSIVE',
        []
    union all
    select
        2200,
        'Pasivos financieros a valor razonable con cambios en resultados (neto) (2230001, 2240002, 2250003)',
        'Financial liabilities at fair value through profit or loss (net) (210001, 210002, 210003)',
        ['2230001', '2240002', '2250003']
    union all
    select
        2600,
        'Derivados para cobertura (2270004)',
        'Derivatives for hedging (210004)',
        ['2270004']
    union all
    select
        2800,
        'Pasivos financieros a costo amortizado (neto) (211)',
        'Financial liabilities at amortized cost (net) (211)',
        ['211']
    union all
    select
        2900,
        '    Depósitos (2110, 2111, 2112, 2113, 2114) (1)',
        '    Deposits (2110, 2111, 2112, 2113, 2114) (1)',
        ['2110', '2111', '2112', '2113', '2114']
    union all
    select
        3000,
        '    Operaciones con pacto de retrocompra (2115)',
        '    Operations with repurchase agreement (2115)',
        ['2115']
    union all
    select
        3100,
        '    Préstamos (2116, 2117, 2118)',
        '    Loans (2116, 2117, 2118)',
        ['2116', '2117', '2118']
    union all
    select
        3200,
        '    Títulos de emisión propia (212001, 212003, 212004)',
        '    Own-issue securities (212001, 212003, 212004)',
        ['212001', '212003', '212004']
    union all
    select
        3300,
        '    Obligaciones convertibles en acciones',
        '    Bonds convertible into shares',
        []
    union all
    select
        3400,
        '    Préstamos convertibles en acciones hasta un año plazo (211611)',
        '    Convertible loans for up to one year (211611)',
        ['211611']
    union all
    select
        3500,
        '    Bonos convertibles en acciones (212002,212005)',
        '    Bonds convertible into shares (212002,212005)',
        ['212002', '212005']
    union all
    select
        3600,
        'Obligaciones a la vista (213)',
        'Demand bonds (213)',
        ['213']
    union all
    select
        3700,
        'Cuentas por pagar (222, 223) (1)',
        'Accounts payable (222 , 223) (1)',
        ['222', '223']
    union all
    select
        3800,
        'Provisiones (2240)',
        'Provisions (2240)',
        ['2240']
    union all
    select
        3900,
        'Otros pasivos (221, 2242,225,4129) (1) (3)',
        'Other liabilities (221, 2242,225, 4129) (1) (3)',
        ['221', '2242', '225', '4129']
    union all
    select
        4000,
        'Préstamos subordinados (2119)',
        'Subordinated loans (2119)',
        ['2119']
    union all
    select
        4100,
        'Total Pasivos',
        'Total Liabilities',
        []
    union all
    select
        4200,
        'PATRIMONIO NETO',
        'NET WORTH',
        []
    union all
    select
        4300,
        'Capital Social (311)4/',
        'Share Capital (311) 4/',
        ['311']
    union all
    select
        4400,
        'Reservas (313)',
        'Reservations (313)',
        ['313']
    union all
    select
        4500,
        '    De capital',
        '    From capital',
        []
    union all
    select
        4600,
        '    Otras reservas',
        '    Other reservations',
        []
    union all
    select
        4700,
        'Resultados por aplicar (314)',
        'Results to be applied (314)',
        ['314']
    union all
    select
        4800,
        '    Utilidades (Pérdidas) de ejercicios anteriores',
        '    Profits (Losses) from previous years',
        []
    union all
    select
        4900,
        '    Utilidades (Pérdidas) del presente ejercicio',
        '    Profits (Losses) for the current fiscal year',
        []
    union all
    select
        5000,
        'Primas sobre acciones (315)',
        'Share premiums (315)',
        ['315']
    union all
    select
        5100,
        'Patrimonio restringido',
        'Restricted assets',
        []
    union all
    select
        5200,
        '    Utilidades no distribuibles (321)',
        '    Non-distributable profits (321)',
        ['321']
    union all
    select
        5300,
        '    Donaciones (322)',
        '    Donations (322)',
        ['322']
    union all
    select
        5400,
        'Otro resultado integral acumulado (3230, 3231)',
        'Accumulated other comprehensive income (3230, 3231)',
        ['3230', '3231']
    union all
    select
        5500,
        '    Elementos que no se reclasificarán a resultados',
        '    Items that will not be reclassified to results',
        []
    union all
    select
        5600,
        '    Elementos que se reclasificarán a resultados',
        '    Items to be reclassified to results',
        []
    union all
    select
        5700,
        'Participaciones no controladoras',
        'Non-controlling interests',
        []
    union all
    select
        5800,
        'Total patrimonio',
        'Total assets',
        []
    union all
    select
        5900,
        'Total Pasivo y Patrimonio',
        'Total Liabilities and Equity',
        []

)

select * from titles
order by order_by
