with

    coefficients as (

        select
            '211001' as account_code,
            'Depósitos a la vista - cuentas corrientes' as account_name,
            'Demand deposits - current accounts' as eng_account_name,
            0.18 as coefficient
        union all
        select
            '211002',
            'Depósitos a la vista - cuentas de ahorro',
            'Demand deposits - savings accounts',
            0.16
        union all
        select
            '211003',
            'Depósitos a la vista - cuentas de ahorro - depósitos en cuenta de ahorro simplificada',
            'Demand deposits - savings accounts - simplified savings account deposits',
            0.16
        union all
        select
            '2111',
            'Depósitos pactados hasta un año plazo',
            'Deposits agreed for up to one year',
            0.12
        union all
        select '211201', 'Depósitos a plazo', 'Term deposits', 0.12
        union all
        select
            '211202',
            'Depósitos a plazo con encaje especial (CEDEVIV y CEDAGRO)',
            'Term deposits with special reserves (CEDEVIV and CEDAGRO)',
            0.10
        union all
        -- --
        -- Dup, needs a more specific account: 211202.XXXX
        -- --
        -- select
        -- '211202',
        -- 'Depósitos a plazo con encaje especial (Para la cancelación de la deuda
        -- agraria y agropecuaria)',
        -- 'Term deposits with special reserves (for the cancellation of agricultural
        -- and livestock debt)',
        -- 0.01,
        -- union all
        select
            '211203',
            'En garantía de cartas de crédito',
            'As a guarantee for letters of credit',
            0.12
        union all
        select '211204', 'De ahorro programado', 'From programmed savings', 0.12
        union all
        select
            '211401',
            'Depósitos restringidos e inactivos - cuentas de ahorro',
            'Restricted and inactive deposits - savings accounts',
            0.16
        union all
        select
            '211402',
            'Depósitos restringidos e inactivos - depósitos a plazo',
            'Restricted and inactive deposits - term deposits',
            0.12
        union all
        select
            '211403',
            'Depósitos embargados - cuentas corrientes',
            'Seized deposits - current accounts',
            0.18
        union all
        select
            '211404',
            'Depósitos embargados - cuenta de ahorro',
            'Seized deposits - savings account',
            0.16
        union all
        select
            '211406',
            'Depósitos inactivos - cuentas corrientes',
            'Inactive deposits - current accounts',
            0.18
        union all
        select
            '211407',
            'Depósitos inactivos - ahorros',
            'Inactive deposits - savings',
            0.16
        union all
        select
            '211408',
            'Depósitos en garantía - cuenta de ahorro simplificada',
            'Escrow deposits - simplified savings account',
            0.16
        union all
        select
            '211409',
            'Depósitos embargados - cuenta de ahorro simplificada',
            'Seized deposits - simplified savings account',
            0.16
        union all
        select
            '211410',
            'Depósitos inactivos - cuenta de ahorro simplificada',
            'Inactive deposits - simplified savings account',
            0.16
        union all
        select
            '2121080101',
            'Adeudado a bancos extranjeros por cartas de crédito',
            'Owed to foreign banks for letters of credit',
            0.03
        union all
        select
            '2121080102',
            'Adeudado a bancos extranjeros por cartas de crédito ME',
            'Owed to foreign banks for ME letters of credit',
            0.03
        union all
        select
            '2121080201',
            'Adeudado a bancos extranjeros por líneas de crédito',
            'Owed to foreign banks for credit lines',
            0.03
        union all
        select
            '2121080202',
            'Adeudado a bancos extranjeros por líneas de crédito ME',
            'Owed to foreign banks for ME credit lines',
            0.03
        union all
        select
            '2121080301',
            'Adeudado a bancos extranjeros - otros',
            'Owed to foreign banks - others',
            0.03
        union all
        select
            '2121080302',
            'Adeudado a bancos extranjeros - otros - ME',
            'Owed to foreign banks - others - ME',
            0.03
        union all
        select
            '2121080501',
            'Adeudado a cooperativas extranjeras *',
            'Owed to foreign cooperatives *',
            0.03
        union all
        select
            '2121080502',
            'Adeudado a cooperativas extranjeras *',
            'Owed to foreign cooperatives *',
            0.03
        union all
        select
            '2121089901',
            'Intereses y otros por pagar',
            'Interest and other payables',
            0.03
        union all
        select
            '2121089902',
            'Intereses y otros por pagar - ME',
            'Interest and other payables - ME',
            0.03
        union all
        select
            '2122080101',
            'Adeudado a bancos extranjeros por cartas de crédito',
            'Owed to foreign banks for letters of credit',
            0.03
        union all
        select
            '2122080102',
            'Adeudado a bancos extranjeros por cartas de crédito ME',
            'Owed to foreign banks for ME letters of credit',
            0.03
        union all
        select
            '2122080201',
            'Adeudado a bancos extranjeros por líneas de crédito',
            'Owed to foreign banks for credit lines',
            0.03
        union all
        select
            '2122080202',
            'Adeudado a bancos extranjeros por líneas de crédito ME',
            'Owed to foreign banks for ME credit lines',
            0.03
        union all
        select
            '2122080301',
            'Adeudado a bancos extranjeros - otros -',
            'Owed to foreign banks - others -',
            0.03
        union all
        select
            '2122080302',
            'Adeudado a bancos extranjeros - otros - ME',
            'Owed to foreign banks - others - ME',
            0.03
        union all
        select
            '2122080501',
            'Adeudado a cooperativas extranjeras *',
            'Owed to foreign cooperatives *',
            0.03
        union all
        select
            '2122080502',
            'Adeudado a cooperativas extranjeras *',
            'Owed to foreign cooperatives *',
            0.03
        union all
        select
            '2122089901',
            'Intereses y otros por pagar',
            'Interest and other payables',
            0.03
        union all
        select
            '2122089902',
            'Intereses y otros por pagar - ME',
            'Interest and other payables - ME',
            0.03
        union all
        select '2130010201', 'Cheques certificados', 'Certified checks', 0.18
        union all
        select '2130010202', 'Cheques certificados - ME', 'Certified Checks - ME', 0.18
        union all
        select
            '2141',
            'Títulos de emisión propia pactados a menos de un año plazo',
            'Own-issue securities agreed for a term of less than one year',
            0.15
        union all
        -- --
        -- Dup, needs a more specific account: 2141.XX.XXXX
        -- --
        -- select
        -- '2141',
        -- 'Títulos de emisión propia a un año plazo',
        -- 'Own-issue securities with a one-year term',
        -- 0.05,
        -- union all
        select
            '2142',
            'Títulos de emisión propia pactados a más de un año plazo (Comprende los pactados a 5 años plazo garantizados con bonos del Estado para la Conversión y Consolidación de la deuda interna garantizada).',
            'Own-issued securities agreed for a term of more than one year (Including those agreed for a term of 5 years guaranteed with government bonds for the conversion and consolidation of guaranteed domestic debt).',
            0.01
        union all
        -- --
        -- Dup, needs a more specific account: 2141.XX.XXXX
        -- --
        -- select
        -- '2142',
        -- 'Títulos de emisión propia pactados a más de un año plazo (Todos los no
        -- comprendidos en la cuenta anterior)',
        -- 'Own-issue securities agreed for a term of more than one year (All those
        -- not included in the previous account)',
        -- 0.05,
        -- union all
        select
            '5120010002',
            'Avales a menos de cinco años plazo ME',
            'Guarantees for less than five years term ME',
            0.05
        union all
        select
            '5120020002',
            'Fianzas a más de cinco años plazo ME',
            'Bonds for more than five years term ME',
            0.05

    )

select *
from coefficients
