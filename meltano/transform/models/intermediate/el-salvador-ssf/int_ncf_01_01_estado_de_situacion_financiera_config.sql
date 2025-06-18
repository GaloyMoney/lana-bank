with

titles as (

  select
    100 as order_by,
    'ACTIVO' as title,
    [] as source_account_codes
    union all
  select
    200,
    'Efectivo y equivalentes de efectivo (111, 112 y todas aquellas partidas que la entidad conforme a sus políticas defina como equivalentes de efectivo)',
    ['111', '112']
    union all
  select
    300,
    'Instrumentos financieros de inversión (neto) (1130, 1131, 1134)',
    ['1130', '1131', '1134']
    union all
  select
    400,
    '    A Valor razonable con cambios en resultados',
    []
    union all
  select
    500,
    '    A Valor razonable con cambios en otro resultado integral (VRORI)',
    []
    union all
  select
    600,
    '    A Costo amortizado',
    []
    union all
  select
    700,
    'Derivados financieros para coberturas (1132)',
    ['1132']
    union all
  select
    800,
    'Instrumentos Financieros Restringidos (1138)',
    ['1138']
    union all
  select
    900,
    'Cartera de créditos (neta) (114)',
    ['114']
    union all
  select
    1000,
    '    Créditos vigentes a un año plazo',
    []
    union all
  select
    1100,
    '    Créditos vigentes a más de un año plazo',
    []
    union all
  select
    1200,
    '    Créditos vencidos',
    []
    union all
  select
    1300,
    '    (Estimación de pérdida por deterioro)',
    []
    union all
  select
    1400,
    'Cuentas por cobrar (neto) (125)',
    ['125']
    union all
  select
    1500,
    'Activos físicos e intangibles (neto) (13)',
    ['13']
    union all
  select
    1600,
    'Activos extraordinarios (neto) (122)',
    ['122']
    union all
  select
    1700,
    'Activos de largo plazo mantenidos para la venta (127)',
    ['127']
    union all
  select
    1800,
    'Inversiones en acciones (Neto) (126)',
    ['126']
    union all
  select
    1900,
    'Otros Activos (121, 123, 124) (1)',
    ['121', '123', '121']
    union all
  select
    2000,
    'Total Activos',
    []
    union all
  select
    2100,
    'PASIVO',
    []
    union all
  select
    2200,
    'Pasivos financieros a valor razonable con cambios en resultados (neto) (2230001, 2240002, 2250003)',
    ['2230001', '2240002', '2250003']
    union all
  select
    2600,
    'Derivados para cobertura (2270004)',
    ['2270004']
    union all
  select
    2800,
    'Pasivos financieros a costo amortizado (neto) (211)',
    ['211']
    union all
  select
    2900,
    '    Depósitos (2110, 2111, 2112, 2113, 2114) (1)',
    ['2110', '2111', '2112', '2113', '2114']
    union all
  select
    3000,
    '    Operaciones con pacto de retrocompra (2115)',
    ['2115']
    union all
  select
    3100,
    '    Préstamos (2116, 2117, 2118)',
    ['2116', '2117', '2118']
    union all
  select
    3200,
    '    Títulos de emisión propia (212001, 212003, 212004)',
    ['212001', '212003', '212004']
    union all
  select
    3300,
    '    Obligaciones convertibles en acciones',
    []
    union all
  select
    3400,
    '    Préstamos convertibles en acciones hasta un año plazo (211611)',
    ['211611']
    union all
  select
    3500,
    '    Bonos convertibles en acciones (212002,212005)',
    ['212002', '212005']
    union all
  select
    3600,
    'Obligaciones a la vista (213)',
    ['213']
    union all
  select
    3700,
    'Cuentas por pagar (222, 223) (1)',
    ['222', '223']
    union all
  select
    3800,
    'Provisiones (2240)',
    ['2240']
    union all
  select
    3900,
    'Otros pasivos (221, 2242,225,4129) (1) (3)',
    ['221', '2242', '225', '4129']
    union all
  select
    4000,
    'Préstamos subordinados (2119)',
    ['2119']
    union all
  select
    4100,
    'Total Pasivos',
    []
    union all
  select
    4200,
    'PATRIMONIO NETO',
    []
    union all
  select
    4300,
    'Capital Social (311)4/',
    ['311']
    union all
  select
    4400,
    'Reservas (313)',
    ['313']
    union all
  select
    4500,
    '    De capital',
    []
    union all
  select
    4600,
    '    Otras reservas',
    []
    union all
  select
    4700,
    'Resultados por aplicar (314)',
    ['314']
    union all
  select
    4800,
    '    Utilidades (Pérdidas) de ejercicios anteriores',
    []
    union all
  select
    4900,
    '    Utilidades (Pérdidas) del presente ejercicio',
    []
    union all
  select
    5000,
    'Primas sobre acciones (315)',
    ['315']
    union all
  select
    5100,
    'Patrimonio restringido',
    []
    union all
  select
    5200,
    '    Utilidades no distribuibles (321)',
    ['321']
    union all
  select
    5300,
    '    Donaciones (322)',
    ['322']
    union all
  select
    5400,
    'Otro resultado integral acumulado (3230, 3231)',
    ['3230', '3231']
    union all
  select
    5500,
    '    Elementos que no se reclasificarán a resultados',
    []
    union all
  select
    5600,
    '    Elementos que se reclasificarán a resultados',
    []
    union all
  select
    5700,
    'Participaciones no controladoras',
    []
    union all
  select
    5800,
    'Total patrimonio',
    []
    union all
  select
    5900,
    'Total Pasivo y Patrimonio',
    []

)

select * from titles
order by order_by
