with

    titles as (

        select
            100 as order_by,
            'SALDOS DE DEPÓSITOS Y OBLIGACIONES SUJETAS DE RESERVA DE LIQUIDEZ'
            as title,
            'DEPOSIT BALANCES AND LIABILITIES SUBJECT TO LIQUIDITY RESERVE'
            as eng_title,
            [] as sum_account_codes,
            [] as diff_account_codes
        union all
        select
            200,
            'a) Dep. Cta. Cte. (211001,211403,211406,2130010201 y 2130010202)',
            'a) Dep. Cta. Cte. (211001, 211403, 211406, 2130010201 and 2130010202)',
            ['211001', '211403', '211406', '2130010201', '2130010202'],
            []
        union all
        select
            300, '1 Sector Público', '1 Public Sector', ['2110010201', '2110010202'], []
        union all
        select
            400,
            '2 Sector Privado',
            '2 Private Sector',
            ['211001', '211403', '211406', '2130010201', '2130010202'],
            ['2110010201', '2110010202']
        union all
        select
            500,
            'b) Dep. Ahorros (211002,211003,211401,211404, 11407,211408,211409 y 211410)',
            'b) Savings Dept. (211002, 211003, 211401, 211404, 11407, 211408, 211409 and 211410)',
            [
                '211002',
                '211003',
                '211401',
                '211404',
                '11407',
                '211408',
                '211409',
                '211410'
            ],
            []
        union all
        select
            600, '1 Sector Público', '1 Public Sector', ['2114010201', '2114010202'], []
        union all
        select
            700,
            '2 Sector Privado',
            '2 Private Sector',
            [
                '211002',
                '211003',
                '211401',
                '211404',
                '11407',
                '211408',
                '211409',
                '211410'
            ],
            ['2114010201', '2114010202']
        union all
        select
            800,
            'c) Dep. a Plazo (2111,2112,211402,211405; excluye: 211202)',
            'c) Term Dep. (2111,2112,211402,211405; excludes: 211202)',
            ['2111', '2112', '211402', '211405'],
            ['211202']
        union all
        select
            900,
            '1 Sector Público',
            '1 Public Sector',
            [
                '2111010201',
                '2111010202',
                '2111020201',
                '2111020202',
                '2111030201',
                '2111030202',
                '2111040201',
                '2111040202',
                '2111050201',
                '2111050202',
                '2111060201',
                '2111060202',
                '2111070201',
                '2111070202',
                '2111080201',
                '2111080202',
                '2111130201',
                '2111130202',
                '2111140201',
                '2111140202',
                '2111990201',
                '2111990202',
                '2112010201',
                '2112010202',
                '2112020201',
                '2112020202',
                '2112030201',
                '2112030202',
                '2112040201',
                '2112040202',
                '2114020201',
                '2114020202',
                '2114050201',
                '2114050202'
            ],
            []
        union all
        select
            1000,
            '2 Sector Privado',
            '2 Private Sector',
            ['2111', '2112', '211402', '211405'],
            [
                '211202',
                '2111010201',
                '2111010202',
                '2111020201',
                '2111020202',
                '2111030201',
                '2111030202',
                '2111040201',
                '2111040202',
                '2111050201',
                '2111050202',
                '2111060201',
                '2111060202',
                '2111070201',
                '2111070202',
                '2111080201',
                '2111080202',
                '2111130201',
                '2111130202',
                '2111140201',
                '2111140202',
                '2111990201',
                '2111990202',
                '2112010201',
                '2112010202',
                '2112020201',
                '2112020202',
                '2112030201',
                '2112030202',
                '2112040201',
                '2112040202',
                '2114020201',
                '2114020202',
                '2114050201',
                '2114050202'
            ]
        union all
        select
            1100,
            'd) Certificados de depósitos a plazo p/vivienda (211202)',
            'd) Certificates of deposit for housing (211202)',
            ['211202'],
            []
        union all
        select
            1200,
            '1 Sector Público',
            '1 Public Sector',
            ['2112020201', '2112020202'],
            []
        union all
        select
            1300,
            '2 Sector Privado',
            '2 Private Sector',
            ['211202'],
            ['2112020201', '2112020202']
        union all
        select
            1400,
            'e) Certificados de depósito a plazo agropecuario (211202)',
            'e) Agricultural term deposit certificates (211202)',
            ['211202'],
            []
        union all
        select
            1500,
            '1 Sector Público',
            '1 Public Sector',
            ['2112020201', '2112020202'],
            []
        union all
        select
            1600,
            '2 Sector Privado',
            '2 Private Sector',
            ['211202'],
            ['2112020201', '2112020202']
        union all
        select
            1700,
            'f) Préstamos adeudados a bancos extranjeros (menores a 5 años)',
            'f) Loans owed to foreign banks (less than 5 years)',
            ['212108', '212208'],
            ['2121080401', '2121080402', '2122080401', '2122080402']
        union all
        select
            1800,
            '1 Hasta un año plazo  (212108, excluye 2121080401 y 2121080402)',
            '1 Up to one year term (212108, excludes 2121080401 and 2121080402)',
            ['212108'],
            ['2121080401', '2121080402']
        union all
        select
            1900,
            '2 Más de un año plazo  (212208, excluye 2122080401 y 2122080402)',
            '2 More than one year term (212208, excludes 2122080401 and 2122080402)',
            ['212208'],
            ['2122080401', '2122080402']
        union all
        select
            2000,
            'g) Títulos de Emisión Propia pactados menos de un año plazo (214100; Excluye los títulos de emisión propia pactados a un año plazo)',
            'g) Own-issue securities agreed for less than one year (214100; Excludes own-issue securities agreed for one year)',
            ['214100'],
            []
        union all
        select 2100, '1 Sector Público', '1 Public Sector', [], []
        union all
        select 2200, '2 Sector Privado', '2 Private Sector', ['214100'], []
        union all
        select
            2300,
            'h) Títulos de Emisión Propia a 1 año plazo y más (214; excluye títulos de emisión propia pactados a menos de un año plazo y Certificados a 5 años o más garantizados con Bonos del Estado para la conversión y Consolidación de la Deuda Int. Garant.)',
            'h) Own Issue Securities with a term of 1 year or more (214; excludes own issue securities agreed for a term of less than one year and Certificates with a term of 5 years or more guaranteed by Government Bonds for the conversion and Consolidation of the International Guarantee Debt)',
            ['214'],
            []
        union all
        select 2400, '1 Sector Público', '1 Public Sector', [], []
        union all
        select 2500, '2 Sector Privado', '2 Private Sector', ['214'], []
        union all
        select
            2600,
            'i) Certificados a 5 años o más, garantizados con Bonos del Estado para la Conversión y Consolidación de la Deuda Interna garantizada (214202)',
            'i) Certificates for 5 years or more, secured by Government Bonds for the Conversion and Consolidation of the Guaranteed Internal Debt (214202)',
            ['214202'],
            []
        union all
        select 2700, '1 Sector Público', '1 Public Sector', [], []
        union all
        select 2800, '2 Sector Privado', '2 Private Sector', ['214202'], []
        union all
        select
            2900,
            'j) Fondos de Fideicomisos recibidos para ser colocados directa o indirectamente en créditos y otros instrumentos financieros (912001)',
            'j) Trust Funds received to be placed directly or indirectly in loans and other financial instruments (912001)',
            ['912001'],
            []
        union all
        select 3000, '1 Sector Público', '1 Public Sector', [], []
        union all
        select 3100, '2 Sector Privado', '2 Private Sector', ['912001'], []
        union all
        select
            3200,
            'k) Certificados de Depósito Especial para Cancelación de Deudas Agrarias y Agropecuarias (211202)',
            'k) Special Deposit Certificates for Cancellation of Agrarian and Agricultural Debts (211202)',
            ['211202'],
            []
        union all
        select
            3300,
            '1 Sector Público',
            '1 Public Sector',
            ['2112020201', '2112020202'],
            []
        union all
        select
            3400,
            '2 Sector Privado',
            '2 Private Sector',
            ['211202'],
            ['2112020201', '2112020202']
        union all
        select
            3500,
            'l) Avales y fianzas con el exterior',
            'l) Guarantees and bonds with foreign countries',
            ['5120010002', '5120020002'],
            []
        union all
        select
            3600,
            '1 Avales (5120010002)',
            '1 Guarantees (5120010002)',
            ['5120010002'],
            []
        union all
        select
            3700, '2 Fianzas (5120020002)', '2 Bonds (5120020002)', ['5120020002'], []
        union all
        select
            3800,
            'Suma de Saldos (a+b+c+d+e+f+g+h+i+j+k+l)',
            'Sum of Balances (a+b+c+d+e+f+g+h+i+j+k+l)',
            [
                '211001',
                '211403',
                '211406',
                '2130010201',
                '2130010202',
                '211002',
                '211003',
                '211401',
                '211404',
                '11407',
                '211408',
                '211409',
                '211410',
                '2111',
                '2112',
                '211402',
                '211405',
                '211202',
                '212108',
                '212208'
            ],
            [
                '2121080401',
                '2121080402',
                '2122080401',
                '2122080402',
                '214100',
                '214',
                '214202',
                '912001',
                '5120010002',
                '5120020002'
            ]
        union all
        select 3900, 'DEPÓSITOS EN BCR', 'DEPOSITS IN BCR', ['111002'], []
        union all
        select 4000, 'RESERVA TOTAL REQUERIDA', 'TOTAL RESERVATION REQUIRED', [], []
        union all
        select
            4100,
            'EXCEDENTE (DEFICIENCIA) DE RESERVA DE LIQUIDEZ',
            'SURPLUS (DEFICIENCY) OF LIQUIDITY RESERVE',
            [],
            []
        union all
        select 4200, 'INFORMACIÓN COMPLEMENTARIA', 'ADDITIONAL INFORMATION', [], []
        union all
        select 4300, 'CUENTAS DE ACTIVO', 'ASSET ACCOUNTS', [], []
        union all
        select
            4400, 'Existencias en Caja (111001)', 'Cash Stock (111001)', ['111001'], []
        union all
        select
            4500,
            'Depósitos en el BCR (111002)',
            'Deposits in the BCR (111002)',
            ['111002'],
            []
        union all
        select
            4600,
            'Documentos a Cargo de Otros Bancos (111003)',
            'Documents Held by Other Banks (111003)',
            ['111003'],
            []
        union all
        select
            4700,
            'Saldo de Inversiones de Reportos (1121)',
            'Balance of Repo Investments (1121)',
            ['1121'],
            []
        union all
        select
            4800,
            'Saldo de Préstamos Brutos otorgados (1141,1142,1148)',
            'Gross Loan Balance Granted (1141,1142,1148)',
            ['1141', '1142', '1148'],
            []
        union all
        select
            4900,
            'Inversiones extranjeras con categoría AAA hasta AA- (113)',
            'Foreign investments with category AAA to AA- (113)',
            ['113'],
            []
        union all
        select
            5000,
            'Inversiones del Ministerio de Hacienda con vencimiento menor a 1 año (113)',
            'Investments of the Ministry of Finance with maturity less than 1 year (113)',
            ['113'],
            []
        union all
        select
            5100,
            'Inversiones del Ministerio de Hacienda con vencimiento mayor a 1 año (113)',
            'Investments of the Ministry of Finance with maturity greater than 1 year (113)',
            ['113'],
            []
        union all
        select
            5200,
            'Depósitos en bancos extranjeros con categoría AAA hasta A- (111006)',
            'Deposits in foreign banks with AAA to A- rating (111006)',
            ['111006'],
            []
        union all
        select
            5300,
            'Depósitos en bancos extranjeros con categoría BBB+ hasta BBB- (111006)',
            'Deposits in foreign banks with ratings BBB+ to BBB- (111006)',
            ['111006'],
            []
        union all
        select
            5400,
            'Inversiones extranjeras con categoría A+ hasta BBB- (113)',
            'Foreign investments with category A+ to BBB- (113)',
            ['113'],
            []
        union all
        select
            5500,
            'Depósitos a plazo en bancos locales (1110040301 y 1110040302)',
            'Term deposits in local banks (1110040301 and 1110040302)',
            ['1110040301', '1110040302'],
            []
        union all
        select
            5600,
            'Depósitos a la vista en bancos locales (1110040101, 1110040102, 1110040201 y 1110040202)',
            'Demand deposits in local banks (1110040101, 1110040102, 1110040201 and 1110040202)',
            ['1110040101', '1110040102', '1110040201', '1110040202'],
            []
        union all
        select
            5700,
            'Préstamos a bancos locales (114105 y 114205)',
            'Loans to local banks (114105 and 114205)',
            ['114105', '114205'],
            []
        union all
        select 5800, 'CUENTAS DE PASIVO', 'LIABILITY ACCOUNTS', [], []
        union all
        select
            5900,
            'Cheques de caja o gerencia (2130010101,2130010102)',
            "Cashier's or manager's checks (2130010101,2130010102)",
            ['2130010101', '2130010102'],
            []
        union all
        select
            6000,
            'Cheques Certificados (2130010201,2130010202)',
            'Certified Checks (2130010201,2130010202)',
            ['2130010201', '2130010202'],
            []
        union all
        select
            6100,
            'Documentos Transados (215)',
            'Transacted Documents (215)',
            ['215'],
            []
        union all
        select
            6200,
            'Títulos de emisión propia pactados a un año plazo',
            'Own-issue securities agreed for a one-year term',
            [],
            []
        union all
        select
            6300,
            'Títulos de emisión propia pactados a 5 años plazo y más sin garantía hipotecaria',
            'Own-issue securities agreed for a term of 5 years or more without mortgage guarantee',
            [],
            []
        union all
        select
            6400,
            'Títulos de emisión propia a 5 años y más, con garantía hipotecaria (exentos de encaje)',
            'Own-issue securities for 5 years or more, with mortgage guarantee (exempt from reserve requirements)',
            [],
            []
        union all
        select
            6500,
            '1 Sector público (214202)',
            '1 Public sector (214202)',
            ['214202'],
            []
        union all
        select
            6600,
            '2 Sector privado (214202)',
            '2 Private sector (214202)',
            ['214202'],
            []
        union all
        select
            6700,
            'Prést. Adeudados a Bancos Ext. A 5 años y más (212308, excluye 2123080401 y 2123080402)',
            'Loans owed to foreign banks for 5 years or more (212308, excluding 2123080401 and 2123080402)',
            ['212308'],
            ['2123080401', '2123080402']
        union all
        select
            6800,
            'Adeudado a bancos locales (212105)',
            'Owed to local banks (212105)',
            ['212105'],
            []
        union all
        select 6900, 'CUENTAS DE CONTINGENCIAS', 'CONTINGENCY ACCOUNTS', [], []
        union all
        select
            7000,
            'Contingencias por Cartas de Crédito de Import. Negociadas (511001,511003)',
            'Contingencies for Negotiated Import Letters of Credit (511001, 511003)',
            [],
            []

    )

select *
from titles
order by order_by
