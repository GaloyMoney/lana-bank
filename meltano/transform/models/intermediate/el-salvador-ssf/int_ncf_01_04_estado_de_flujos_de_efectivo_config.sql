with

titles as (

    select
        100 as order_by,
        '    A. Flujos de efectivo proveniente de actividades de operación:' as title,
        [] as source_account_codes,
        union all
    select
        200,
        'Utilidad neta del período (1)',
        [],
        union all
    select
        300,
        'Ajustes para conciliar la utilidad neta con el efectivo de las actividades de operación: (1)',
        [],
        union all
    select
        400,
        'Reservas para saneamientos de activos de riesgo crediticio (1)',
        [],
        union all
    select
        500,
        'Reservas de saneamiento de otros activos (1)',
        [],
        union all
    select
        600,
        'Participación en asociadas (1)',
        [],
        union all
    select
        700,
        'Depreciaciones (1)',
        [],
        union all
    select
        800,
        'Amortizaciones (1)',
        [],
        union all
    select
        900,
        'Resultados en venta y/o retiro de activos extraordinarios (1)',
        [],
        union all
    select
        1000,
        'Resultados en venta y/o retiro de activos físicos (1)',
        [],
        union all
    select
        1100,
        'Participación del interés minoritario (1)',
        [],
        union all
    select
        1200,
        'Intereses y comisiones por recibir (1)',
        [],
        union all
    select
        1300,
        'Intereses y comisiones por pagar (1)',
        [],
        union all
    select
        1400,
        'Variación en cuentas de activos: (1)',
        [],
        union all
    select
        1500,
        '(Incrementos) disminuciones en Préstamos',
        [],
        union all
    select
        1600,
        '(Incrementos) disminuciones en Cuentas por cobrar (1)',
        [],
        union all
    select
        1700,
        'Ventas de Activos extraordinarios (1)',
        [],
        union all
    select
        1800,
        '(Incrementos) disminuciones en otros activos (1)',
        [],
        union all
    select
        1900,
        'Variación en cuentas de pasivos: (1)',
        [],
        union all
    select
        2000,
        'Incrementos (disminuciones) en Depósitos (1)',
        [],
        union all
    select
        2100,
        'Incrementos (disminuciones) en Títulos de emisión propia (1)',
        [],
        union all
    select
        2200,
        'Incrementos (disminuciones) en Obligaciones a la vista (1)',
        [],
        union all
    select
        2300,
        'Incrementos (disminuciones) en Cuentas por pagar (1)',
        [],
        union all
    select
        2400,
        'Incrementos (disminuciones) Otros pasivos (1)',
        [],
        union all
    select
        2500,
        'Efectivo neto usado en las actividades de operación (1)',
        [],
        union all
    select
        2600,
        '    B. Flujos de efectivo proveniente de actividades de inversión (1)',
        [],
        union all
    select
        2700,
        '(Incrementos) disminuciones en Instrumentos financieros de inversión (1)',
        [],
        union all
    select
        2800,
        'Adquisición de subsidiarias neto de efectivo adquirido',
        [],
        union all
    select
        2900,
        'Desapropiación de subsidiarias neto de efectivo desapropiado',
        [],
        union all
    select
        3000,
        'Adquisición de activos físicos',
        [],
        union all
    select
        3100,
        'Ingresos por venta de activos físicos',
        [],
        union all
    select
        3200,
        'Adquisición de intangibles',
        [],
        union all
    select
        3300,
        'Ingresos por venta de activos intangibles',
        [],
        union all
    select
        3400,
        'Adquisición de participación en negocios conjuntos',
        [],
        union all
    select
        3500,
        'Beneficios de la venta de participación en negocios conjuntos',
        [],
        union all
    select
        3600,
        'Efectivo neto (usado en) provisto por las actividades de inversión',
        [],
        union all
    select
        3700,
        '    C. Flujos de efectivo proveniente de actividades de financiamiento (1)',
        [],
        union all
    select
        3800,
        'Incrementos de capital social',
        [],
        union all
    select
        3900,
        'Disminuciones de capital social',
        [],
        union all
    select
        4000,
        'Préstamos recibidos',
        [],
        union all
    select
        4100,
        'Pagos de Préstamos',
        [],
        union all
    select
        4200,
        'Colocación de Títulos de emisión propia (1)',
        [],
        union all
    select
        4300,
        'Cancelación de títulos de emisión propia (1)',
        [],
        union all
    select
        4400,
        'Incrementos (disminuciones) Operaciones con pacto de retrocompra (1)',
        [],
        union all
    select
        4500,
        'Pago de arrendamientos financieros',
        [],
        union all
    select
        4600,
        'Pago de dividendos',
        [],
        union all
    select
        4700,
        'Otras actividades de financiamiento',
        [],
        union all
    select
        4800,
        'Efectivo neto provisto (usado) en actividades de financiamiento',
        [],
        union all
    select
        4900,
        'Incremento (Disminución) Neto en el efectivo y equivalentes de efectivo',
        [],
        union all
    select
        5000,
        'Efectivo y Equivalente de Efectivo al 01 de enero',
        [],
        union all
    select
        5100,
        'Efectivo neto proveído (utilizado) por las actividades de operación',
        [],
        union all
    select
        5200,
        'Efectivo neto proveído (utilizado) por las actividades de inversión',
        [],
        union all
    select
        5300,
        'Efectivo neto proveído (utilizado) por las actividades de financiamiento',
        [],
        union all
    select
        5400,
        'Efecto de las fluctuaciones de la tasa de cambio en el efectivo y el equivalente de efectivo poseído',
        [],
        union all
    select
        5500,
        'Efectivo y equivalentes de efectivo al 31 de diciembre',
        [],

)

select * from titles
order by order_by
