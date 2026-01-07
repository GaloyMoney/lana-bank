with

    titles as (

        select
            100 as order_by,
            'Ingresos por intereses' as title,
            'Interest income' as eng_title,
            [] as source_account_spaced_codes
        union all
        select
            200,
            ' Activos financieros a valor razonable con cambios en resultados (6110 02 0100, 6110 02 0200, 6110 02 0300)',
            ' Financial assets at fair value through profit or loss (6110 02 0100, 6110 02 0200, 6110 02 0300)',
            ['6110 02 0100', '6110 02 0200', '6110 02 0300']
        union all
        select
            300,
            ' Activos financieros a valor razonable con cambios en otro resultado integral (6110 02 0400)',
            'Financial assets at fair value through other comprehensive income (6110 02 0400)',
            ['6110 02 0400']
        union all
        select
            400,
            ' Activos financieros a costo amortizado (6110 02 0500, 6110 02 0600, 6110 03 0100,611004)',
            'Financial assets at amortized cost (6110 02 0500, 6110 02 0600, 6110 03 0100,611004)',
            ['6110 02 0500', '6110 02 0600', '6110 03 0100', '611004']
        union all
        select
            500,
            '         Cartera de préstamos (6110 01 0100)',
            '         Loan portfolio (6110 01 0100)',
            ['6110 01 0100']
        union all
        select
            600,
            '         Otros ingresos por intereses (6110 03 0200, 6210 01 0300)',
            '         Other interest income (6110 03 0200, 6210 01 0300 )',
            ['6110 03 0200', '6210 01 0300']
        union all
        select 700, '(Gastos por intereses)', '(Interest expenses)', []
        union all
        select
            800,
            '       (Depósitos) (7110 01)',
            '       (Deposits) (7110 01)',
            ['7110 01']
        union all
        select
            900,
            '       (Pasivos financieros a valor razonable con cambios en resultados) (7110 05)',
            '       (Financial liabilities at fair value through profit or loss) (7110 05)',
            ['7110 05']
        union all
        select
            1000,
            '       (Títulos de emisión propia) (7110 04 0100)',
            '       (Self-issued titles) (7110 04 0100)',
            ['7110 04 0100']
        union all
        select
            1100,
            '       (Préstamos) (7110 02 0100,7110 03 0100)',
            '       (Loans) (7110 02 0100,7110 03 0100)',
            ['7110 02 0100', '7110 03 0100']
        union all
        select
            1200,
            '(Otros gastos por intereses) (7110 07 0100, 711008, 7110 09 0100, 7110 10)',
            '(Other interest expenses) (7110 07 0100, 711008, 7110 09 0100, 7110 10)',
            ['7110 07 0100', '711008', '7110 09 0100', '7110 10']
        union all
        select 1300, 'INGRESOS POR INTERESES NETOS', 'NET INTEREST INCOME', []
        union all
        select
            1400,
            'Ganancia (Pérdida) por cambios en el valor razonable de activos y pasivos financieros, Neta (6111, 7111)',
            'Gain (Loss) from changes in the fair value of financial assets and liabilities, Net (6111, 7111)',
            ['6111', '7111']
        union all
        select
            1500,
            'Ganancia (Pérdida) deterioro de activos financieros distintos a los activos de riesgo crediticio, Neta (6112 01, 7112 01)',
            'Gain (Loss) impairment of financial assets other than credit risk assets, Net (6112 01, 7112 01)',
            ['6112 01', '7112 01']
        union all
        select
            1600,
            'Ganancia (Pérdida) deterioro de activos financieros de riesgo crediticio, Neta (6112 02, 7112 02, 7120) (1)',
            'Gain (Loss) impairment of credit risk financial assets, Net (6112 02, 7112 02, 7120) (1)',
            ['6112 02', '7112 02', '7120']
        union all
        select
            1700,
            'Ganancia o (Pérdida) por reversión de (deterioro) de valor de activos extraordinarios, Neta (6310 09 0300, 7214 01)',
            'Gain or (Loss) on reversal of (impairment) of extraordinary assets, Net (6310 09 0300, 7214 01)',
            ['6310 09 0300', '7214 01']
        union all
        select
            1800,
            'Ganancia (Pérdida) por reversión de (deterioro) de valor de propiedades y equipo, Neta (6310 09 0200, 7214 02)',
            'Gain (Loss) on reversal of (impairment) of property and equipment, Net (6310 09 0200, 7214 02)',
            ['6310 09 0200', '7214 02']
        union all
        select
            1900,
            'Ganancia (Pérdida) por reversión de (deterioro) de otros activos, Neta (6310 09 0100, 6310 09 0400-(7214 03, 7214 04))',
            'Gain (Loss) on reversal of (impairment) of other assets, Net (6310 09 0100, 6310 09 0400-(7214 03, 7214 04))',
            ['6310 09 0100', '7214 03', '7214 04']
        union all
        select
            2000,
            'INGRESOS INTERESES, DESPUÉS DE CARGOS POR DETERIORO',
            'INTEREST INCOME, AFTER IMPAIRMENT CHARGES',
            []
        union all
        select
            2100,
            'Ingresos por comisiones y honorarios (6110 01 0200/6110 01 0700, 6210 01 0200, 6210 03 0100, 6210 04/ 6210 08, 6210 10) (2)',
            'Income from commissions and fees (6110 01 0200/6110 01 0700, 6210 01 0200, 6210 03 0100, 6210 04 / 6210 08, 6210 10) (2)',
            [
                '6110 01 0200',
                '6110 01 0700',
                '6210 01 0200',
                '6210 03 0100',
                '6210 04',
                '6210 08',
                '6210 10'
            ]
        union all
        select
            2200,
            '(Gastos por comisiones y honorarios) (7110 02 0200, 7110 03 0200, 7110 07 02 00,7110 09 0200,7110 11, 7110 13, 7210)',
            '(Commission and fee expenses) (7110 02 0200, 7110 03 0200, 7110 07 02 00,7110 09 0200,7110 11, 7110 13, 7210)',
            [
                '7110 02 0200',
                '7110 03 0200',
                '7110 07 02 00',
                '7110 09 0200',
                '7110 11',
                '7110 13',
                '7210'
            ]
        union all
        select
            2300,
            'INGRESOS POR COMISIONES Y HONORARIOS, NETOS',
            'COMMISSION AND FEES INCOME, NET',
            []
        union all
        select
            2400,
            'Ganancias (Pérdidas) por ventas o desapropiación de instrumentos financieros a costo amortizado, neto (6113 01,7113 01)',
            'Gains (Losses) from sales or disposals of financial instruments at amortized cost, net (6113 01,7113 01)',
            ['6113 01', '7113 01']
        union all
        select
            2500,
            'Ganancia (Pérdida) por ventas de activos y Operaciones discontinuadas (6310 02, 7215)',
            'Gain (Loss) on sales of assets and discontinued operations (6310 02, 7215)',
            ['6310 02', '7215']
        union all
        select
            2600,
            'Ganancias (pérdidas) generadas por entidades registradas bajo el método de la participación (6310 07, 7211)',
            'Profits (losses) generated by entities registered under the equity method (6310 07, 7211)',
            ['6310 07', '7211']
        union all
        select
            2700,
            'Otros ingresos (gastos) financieros (6210 01 0100, 6210 01 0400, 6210 01 0500, 6210 02, 6210 03 9700, 6210 09, 6310 01, 6310 03/6310 06, 6310 08, 6310 99), (7110 04 0200, 7110 06, 7112 03, 7112 04, 7114, 7212, 7213, 7299) (1) (2)',
            'Other financial income (expenses) (6210 01 0100, 6210 01 0400, 6210 01 0500, 6210 02, 6210 03 9700, 6210 09, 6310 01 , 6310 03/6310 06, 6310 08, 6310 99), ( 7110 04 0200, 7110 06, 7112 03, 7112 04, 7114, 7212, 7213, 7299) (1) (2)',
            [
                '6210 01 0100',
                '6210 01 0400',
                '6210 01 0500',
                '6210 02',
                '6210 03 9700',
                '6210 09',
                '6310 01',
                '6310 03',
                '6310 06',
                '6310 08',
                '6310 99',
                '7110 04 0200',
                '7110 06',
                '7112 03',
                '7112 04',
                '7114',
                '7212',
                '7213',
                '7299'
            ]
        union all
        select 2800, 'TOTAL INGRESOS NETOS', 'TOTAL NET INCOME', []
        union all
        select 2900, '(Gastos de administración)', '(Administrative expenses)', []
        union all
        select
            3000,
            '      (Gastos de funcionarios y empleados) (8110)',
            '      (Expenses of officials and employees) (8110)',
            ['8110']
        union all
        select 3100, 'Gastos generales) (8120)', 'General expenses) (8120)', ['8120']
        union all
        select
            3200,
            'Gastos de depreciación y amortización) (8130)',
            'Depreciation and amortization expenses) (8130)',
            ['8130']
        union all
        select
            3300,
            '      (Gastos por provisiones) (8140)',
            '      (Provision expenses) (8140)',
            ['8140']
        union all
        select
            3400,
            'UTILIDAD (PÉRDIDA) ANTES DE IMPUESTO',
            'PROFIT ( LOSS ) BEFORE TAX',
            []
        union all
        select
            3500,
            'Gastos por impuestos sobre las ganancias (8150)',
            'Income tax expenses (8150)',
            ['8150']
        union all
        select
            3600, 'UTILIDAD (PÉRDIDA) DEL EJERCICIO', 'PROFIT (LOSS) FOR THE YEAR  ', []
        union all
        select 3700, 'OTRO RESULTADO INTEGRAL', 'OTHER COMPREHENSIVE INCOME', []
        union all
        select
            3800,
            'Elementos que no se reclasificaran en resultados',
            'Items that will not be reclassified in results',
            []
        union all
        select 3900, 'Superávit por revaluación', 'Revaluation surplus', []
        union all
        select
            4000,
            'Cambios de valor razonable de los pasivos financieros a valor razonable con cambios en resultados atribuibles a cambios en el riesgo de crédito',
            'Fair value changes of financial liabilities at fair value through profit or loss attributable to changes in credit risk',
            []
        union all
        select
            4100,
            'Cambios en el valor razonable del valor temporal de una opción de una partida cubierta relacionada con una transacción',
            'Changes in the fair value of the time value of an option on a hedged item related to a transaction',
            []
        union all
        select
            4200,
            'Cambios en el valor razonable del elemento a término de los contratos a término de una partida cubierta relacionada con una transacción',
            "Changes in the fair value of the forward element of a hedged item's forward contracts related to a transaction",
            []
        union all
        select
            4300,
            'Impuestos de los elementos que no se reclasificaran en resultados',
            'Taxes on items that will not be reclassified in results',
            []
        union all
        select
            4400,
            'Elementos que se reclasificaran en resultados',
            'Items to be reclassified in results',
            []
        union all
        select
            4500,
            'Diferencias de conversión de negocio en el extranjero',
            'Foreign business conversion differences',
            []
        union all
        select
            4600,
            'Reserva de cobertura de flujos de efectivo',
            'Cash flow hedge reserve',
            []
        union all
        select
            4700,
            'Cambios en el valor razonable de instrumentos de deuda a valor razonable con cambios en Otro Resultado Integral.',
            'Changes in the fair value of debt instruments at fair value through other comprehensive income.',
            []
        union all
        select
            4800,
            'Cambios en el valor razonable del valor temporal de una opción de una partida cubierta relacionada con una transacción',
            'Changes in the fair value of the time value of an option on a hedged item related to a transaction',
            []
        union all
        select
            4900,
            'Cambios en el valor razonable del valor temporal de una opción de una partida cubierta relacionada con un período de tiempo',
            "Changes in the fair value of the time value of a hedged item's option related to a time period",
            []
        union all
        select
            5000,
            'Cambios en el valor razonable del elemento a término de los contratos a término de una partida cubierta relacionada con una transacción',
            "Changes in the fair value of the forward element of a hedged item's forward contracts related to a transaction",
            []
        union all
        select
            5100,
            'Cambios en el valor razonable del elemento a término de los contratos a término de una partida cubierta relacionada con un período de tiempo',
            "Changes in the fair value of the forward element of a hedged item's forward contracts relate to a period of time",
            []
        union all
        select
            5200,
            'Impuestos de los elementos que se reclasificaran en resultados',
            'Taxes on items to be reclassified in results',
            []
        union all
        select
            5300,
            'RESULTADO INTEGRAL TOTAL DEL EJERCICIO',
            'TOTAL COMPREHENSIVE INCOME FOR THE YEAR',
            []
        union all
        select
            5400,
            'Ganancia por Acción de las operaciones que continúan atribuible a los accionistas de la matriz durante el año (expresada en ___por acción):',
            'Earnings per share from continuing operations attributable to parent company shareholders for the year (expressed as ___ per share):',
            []
        union all
        select 5500, '  Básica', '  Basic', []
        union all
        select 5600, '  Diluida', '  Diluted', []
        union all
        select
            5700,
            'Ganancia por Acción de las operaciones discontinuadas atribuible a los accionistas de la matriz durante el año (expresada en ___por acción):',
            'Earnings per share from discontinued operations attributable to parent stockholders during the year (expressed as ___ per share):',
            []
        union all
        select 5800, '  Básica', '  Basic', []
        union all
        select 5900, '  Diluida', '  Diluted', []

    )

select *
from titles
order by order_by
