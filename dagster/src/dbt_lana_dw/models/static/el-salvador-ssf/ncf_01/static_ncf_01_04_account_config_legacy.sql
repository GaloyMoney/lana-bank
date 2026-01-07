with

    titles as (

        select
            100 as order_by,
            '    A. Flujos de efectivo proveniente de actividades de operación:'
            as title,
            '    A. Cash flows from operating activities:' as eng_title,
            cast([] as array<string>) as source_account_codes
        union all
        select 200, 'Utilidad neta del período (1)', 'Net income for the period (1)', []
        union all
        select
            300,
            'Ajustes para conciliar la utilidad neta con el efectivo de las actividades de operación: (1)',
            'Adjustments to reconcile net income to cash from operating activities: (1)',
            []
        union all
        select
            400,
            'Reservas para saneamientos de activos de riesgo crediticio (1)',
            'Reserves for write-offs of credit risk assets (1)',
            []
        union all
        select
            500,
            'Reservas de saneamiento de otros activos (1)',
            'Reserves for the write-off of other assets (1)',
            []
        union all
        select
            600, 'Participación en asociadas (1)', 'Participation in associates (1)', []
        union all
        select 700, 'Depreciaciones (1)', 'Depreciations (1)', []
        union all
        select 800, 'Amortizaciones (1)', 'Amortizations (1)', []
        union all
        select
            900,
            'Resultados en venta y/o retiro de activos extraordinarios (1)',
            'Results from the sale and/or withdrawal of extraordinary assets (1)',
            []
        union all
        select
            1000,
            'Resultados en venta y/o retiro de activos físicos (1)',
            'Results from the sale and/or withdrawal of physical assets (1)',
            []
        union all
        select
            1100,
            'Participación del interés minoritario (1)',
            'Minority interest participation (1)',
            []
        union all
        select
            1200,
            'Intereses y comisiones por recibir (1)',
            'Interest and commissions receivable (1)',
            []
        union all
        select
            1300,
            'Intereses y comisiones por pagar (1)',
            'Interest and commissions payable (1)',
            []
        union all
        select
            1400,
            'Variación en cuentas de activos: (1)',
            'Variation in asset accounts: (1)',
            []
        union all
        select
            1500,
            '(Incrementos) disminuciones en Préstamos',
            '(Increases) decreases in Loans',
            []
        union all
        select
            1600,
            '(Incrementos) disminuciones en Cuentas por cobrar (1)',
            '(Increases) decreases in Accounts Receivable (1)',
            []
        union all
        select
            1700,
            'Ventas de Activos extraordinarios (1)',
            'Sales of extraordinary assets (1)',
            []
        union all
        select
            1800,
            '(Incrementos) disminuciones en otros activos (1)',
            '(Increases) decreases in other assets (1)',
            []
        union all
        select
            1900,
            'Variación en cuentas de pasivos: (1)',
            'Variation in liability accounts: (1)',
            []
        union all
        select
            2000,
            'Incrementos (disminuciones) en Depósitos (1)',
            'Increases (decreases) in Deposits (1)',
            []
        union all
        select
            2100,
            'Incrementos (disminuciones) en Títulos de emisión propia (1)',
            'Increases (decreases) in own-issue securities (1)',
            []
        union all
        select
            2200,
            'Incrementos (disminuciones) en Obligaciones a la vista (1)',
            'Increases (decreases) in Demand Bonds (1)',
            []
        union all
        select
            2300,
            'Incrementos (disminuciones) en Cuentas por pagar (1)',
            'Increases (decreases) in Accounts Payable (1)',
            []
        union all
        select
            2400,
            'Incrementos (disminuciones) Otros pasivos (1)',
            'Increases (decreases) Other liabilities (1)',
            []
        union all
        select
            2500,
            'Efectivo neto usado en las actividades de operación (1)',
            'Net cash used in operating activities (1)',
            []
        union all
        select
            2600,
            '    B. Flujos de efectivo proveniente de actividades de inversión (1)',
            '    B. Cash flows from investing activities (1)',
            []
        union all
        select
            2700,
            '(Incrementos) disminuciones en Instrumentos financieros de inversión (1)',
            '(Increases) decreases in Investment financial instruments (1)',
            []
        union all
        select
            2800,
            'Adquisición de subsidiarias neto de efectivo adquirido',
            'Acquisition of subsidiaries net of cash acquired',
            []
        union all
        select
            2900,
            'Desapropiación de subsidiarias neto de efectivo desapropiado',
            'Divestiture of subsidiaries net of cash disappropriated',
            []
        union all
        select
            3000, 'Adquisición de activos físicos', 'Acquisition of physical assets', []
        union all
        select
            3100,
            'Ingresos por venta de activos físicos',
            'Proceeds from the sale of physical assets',
            []
        union all
        select 3200, 'Adquisición de intangibles', 'Acquisition of intangibles', []
        union all
        select
            3300,
            'Ingresos por venta de activos intangibles',
            'Income from the sale of intangible assets',
            []
        union all
        select
            3400,
            'Adquisición de participación en negocios conjuntos',
            'Acquisition of interests in joint ventures',
            []
        union all
        select
            3500,
            'Beneficios de la venta de participación en negocios conjuntos',
            'Benefits of selling interests in joint ventures',
            []
        union all
        select
            3600,
            'Efectivo neto (usado en) provisto por las actividades de inversión',
            'Net cash (used in) provided by investing activities',
            []
        union all
        select
            3700,
            '    C. Flujos de efectivo proveniente de actividades de financiamiento (1)',
            '    C. Cash flows from financing activities (1)',
            []
        union all
        select 3800, 'Incrementos de capital social', 'Share capital increases', []
        union all
        select 3900, 'Disminuciones de capital social', 'Decreases in share capital', []
        union all
        select 4000, 'Préstamos recibidos', 'Loans received', []
        union all
        select 4100, 'Pagos de Préstamos', 'Loan Payments', []
        union all
        select
            4200,
            'Colocación de Títulos de emisión propia (1)',
            'Placement of own-issue securities (1)',
            []
        union all
        select
            4300,
            'Cancelación de títulos de emisión propia (1)',
            'Cancellation of self-issued securities (1)',
            []
        union all
        select
            4400,
            'Incrementos (disminuciones) Operaciones con pacto de retrocompra (1)',
            'Increases (decreases) Operations with repurchase agreement (1)',
            []
        union all
        select
            4500,
            'Pago de arrendamientos financieros',
            'Payment of financial leases',
            []
        union all
        select 4600, 'Pago de dividendos', 'Payment of dividends', []
        union all
        select
            4700,
            'Otras actividades de financiamiento',
            'Other financing activities',
            []
        union all
        select
            4800,
            'Efectivo neto provisto (usado) en actividades de financiamiento',
            'Net cash provided (used) in financing activities',
            []
        union all
        select
            4900,
            'Incremento (Disminución) Neto en el efectivo y equivalentes de efectivo',
            'Net Increase (Decrease) in Cash and Cash Equivalents',
            []
        union all
        select
            5000,
            'Efectivo y Equivalente de Efectivo al 01 de enero',
            'Cash and Cash Equivalents as of January 1',
            []
        union all
        select
            5100,
            'Efectivo neto proveído (utilizado) por las actividades de operación',
            'Net cash provided (used) by operating activities',
            []
        union all
        select
            5200,
            'Efectivo neto proveído (utilizado) por las actividades de inversión',
            'Net cash provided (used) by investing activities',
            []
        union all
        select
            5300,
            'Efectivo neto proveído (utilizado) por las actividades de financiamiento',
            'Net cash provided (used) by financing activities',
            []
        union all
        select
            5400,
            'Efecto de las fluctuaciones de la tasa de cambio en el efectivo y el equivalente de efectivo poseído',
            'Effect of exchange rate fluctuations on cash and cash equivalents held',
            []
        union all
        select
            5500,
            'Efectivo y equivalentes de efectivo al 31 de diciembre',
            'Cash and cash equivalents as of December 31',
            []

    )

select *
from titles
order by order_by
