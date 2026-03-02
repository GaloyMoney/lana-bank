---
id: financial-reports
title: Informes Financieros
sidebar_position: 2
---

# Informes Financieros

Este documento describe los informes financieros disponibles, su estructura y cómo generarlos.

## Balanza de Comprobación

La balanza de comprobación muestra los saldos de todas las cuentas contables en un período determinado.

### Estructura

| Columna | Descripción |
|---------|-------------|
| Cuenta | Código y nombre de cuenta |
| Débito | Total de movimientos débito |
| Crédito | Total de movimientos crédito |
| Saldo | Saldo resultante |

### Generación

#### Desde el Panel

1. Navegar a **Reportes** > **Balanza de Comprobación**
2. Seleccionar período:
   - Fecha inicio
   - Fecha fin
3. Hacer clic en **Generar**

#### Via API GraphQL

```graphql
query GetTrialBalance($input: TrialBalanceInput!) {
  trialBalance(input: $input) {
    accounts {
      code
      name
      debit
      credit
      balance
    }
    totals {
      debit
      credit
    }
    asOfDate
  }
}
```

### Ejemplo de Salida

```
BALANZA DE COMPROBACIÓN
Al 31 de Diciembre de 2024

Cuenta                          Débito        Crédito       Saldo
─────────────────────────────────────────────────────────────────
1000 - Efectivo               100,000.00                 100,000.00
1100 - Préstamos              500,000.00                 500,000.00
2000 - Depósitos Clientes                   400,000.00  -400,000.00
3000 - Capital                              150,000.00  -150,000.00
4000 - Ingresos por Intereses                50,000.00   -50,000.00
─────────────────────────────────────────────────────────────────
TOTALES                       600,000.00    600,000.00          0.00
```

## Balance General

El balance general presenta la posición financiera de la institución en una fecha determinada.

### Estructura

```
┌─────────────────────────────────────────────────────────────────┐
│                    BALANCE GENERAL                              │
│                                                                  │
│  ACTIVOS                          │  PASIVOS Y PATRIMONIO       │
│  ─────────────────────────────────┼───────────────────────────  │
│  Activos Corrientes              │  Pasivos Corrientes          │
│    Efectivo                      │    Depósitos                 │
│    Préstamos a Corto Plazo       │    Obligaciones              │
│                                  │                              │
│  Activos No Corrientes           │  Pasivos No Corrientes       │
│    Préstamos a Largo Plazo       │    Deuda a Largo Plazo       │
│    Activos Fijos                 │                              │
│                                  │  Patrimonio                  │
│                                  │    Capital                   │
│                                  │    Utilidades Retenidas      │
└─────────────────────────────────────────────────────────────────┘
```

### Generación

```graphql
query GetBalanceSheet($asOfDate: Date!) {
  balanceSheet(asOfDate: $asOfDate) {
    assets {
      current {
        cash
        shortTermLoans
        total
      }
      nonCurrent {
        longTermLoans
        fixedAssets
        total
      }
      total
    }
    liabilities {
      current {
        deposits
        shortTermObligations
        total
      }
      nonCurrent {
        longTermDebt
        total
      }
      total
    }
    equity {
      capital
      retainedEarnings
      total
    }
    totalLiabilitiesAndEquity
  }
}
```

## Estado de Resultados

El estado de resultados muestra los ingresos, gastos y utilidad de un período.

### Estructura

| Sección | Componentes |
|---------|-------------|
| Ingresos | Intereses por préstamos, comisiones |
| Gastos Financieros | Intereses pagados |
| Gastos Operativos | Salarios, administración |
| Utilidad Neta | Ingresos - Gastos |

### Generación

```graphql
query GetIncomeStatement($input: IncomeStatementInput!) {
  incomeStatement(input: $input) {
    revenue {
      interestIncome
      feeIncome
      otherIncome
      total
    }
    expenses {
      interestExpense
      operatingExpenses
      provisions
      total
    }
    netIncome
    period {
      start
      end
    }
  }
}
```

## Reportes de Cartera

### Cartera de Crédito

Detalle de todas las líneas de crédito activas.

```graphql
query GetCreditPortfolioReport($asOfDate: Date!) {
  creditPortfolioReport(asOfDate: $asOfDate) {
    facilities {
      id
      customer {
        name
      }
      principal
      outstanding
      status
      interestRate
      maturityDate
    }
    summary {
      totalFacilities
      totalPrincipal
      totalOutstanding
      averageRate
    }
  }
}
```

### Reporte de Morosidad

Análisis de cartera por días de atraso.

| Categoría | Descripción |
|-----------|-------------|
| Al día | Sin atraso |
| 1-30 días | Atraso menor |
| 31-60 días | Atraso moderado |
| 61-90 días | Atraso significativo |
| > 90 días | Cartera vencida |

```graphql
query GetDelinquencyReport($asOfDate: Date!) {
  delinquencyReport(asOfDate: $asOfDate) {
    buckets {
      category
      count
      principal
      percentage
    }
    total {
      facilities
      principal
    }
  }
}
```

## Reportes de Colateral

### Valoración de Colateral

```graphql
query GetCollateralValuationReport($asOfDate: Date!) {
  collateralValuationReport(asOfDate: $asOfDate) {
    collateral {
      facilityId
      type
      originalValue
      currentValue
      ltv
      lastValuationDate
    }
    summary {
      totalCollateralValue
      averageLTV
      collateralCoverage
    }
  }
}
```

## Reportes Regulatorios

### Concentración de Crédito

Análisis de exposición por cliente o sector.

```graphql
query GetConcentrationReport($asOfDate: Date!) {
  concentrationReport(asOfDate: $asOfDate) {
    byCustomer {
      customer
      exposure
      percentageOfPortfolio
    }
    bySector {
      sector
      exposure
      percentageOfPortfolio
    }
    largestExposures {
      customer
      exposure
    }
  }
}
```

### Ratios de Capital

```graphql
query GetCapitalRatios($asOfDate: Date!) {
  capitalRatios(asOfDate: $asOfDate) {
    tier1Capital
    tier2Capital
    riskWeightedAssets
    capitalAdequacyRatio
    leverageRatio
    liquidity {
      lcr
      nsfr
    }
  }
}
```

## Exportación de Reportes

### Formatos Disponibles

| Formato | Extensión | Uso |
|---------|-----------|-----|
| PDF | .pdf | Presentación formal |
| Excel | .xlsx | Análisis |
| CSV | .csv | Integración |
| JSON | .json | APIs |

### Via API

```graphql
mutation ExportReport($input: ReportExportInput!) {
  reportExport(input: $input) {
    downloadUrl
    expiresAt
    format
  }
}
```

## Programación de Reportes

### Configurar Reporte Automático

```graphql
mutation ScheduleReport($input: ReportScheduleInput!) {
  reportSchedule(input: $input) {
    schedule {
      id
      reportType
      frequency
      nextRun
      recipients
    }
  }
}
```

### Frecuencias Disponibles

| Frecuencia | Descripción |
|------------|-------------|
| DAILY | Todos los días |
| WEEKLY | Semanal |
| MONTHLY | Mensual |
| QUARTERLY | Trimestral |
| YEARLY | Anual |

## Permisos Requeridos

| Operación | Permiso |
|-----------|---------|
| Ver reportes financieros | REPORT_FINANCIAL_READ |
| Ver reportes de cartera | REPORT_PORTFOLIO_READ |
| Ver reportes regulatorios | REPORT_REGULATORY_READ |
| Exportar reportes | REPORT_EXPORT |
| Programar reportes | REPORT_SCHEDULE |

## Recorrido en Panel de Administración: Balanza de Comprobación

**Paso 1.** Abre el reporte de balanza de comprobación.

![Balanza de comprobación](/img/screenshots/current/es/trial-balance.cy.ts/trial-balance.png)

**Paso 2.** Cambia la moneda de visualización (ejemplo: BTC).

![Balanza BTC](/img/screenshots/current/es/trial-balance.cy.ts/trial-balance-btc-currency.png)

## Recorrido en Panel de Administración: Balance General

**Paso 1.** Abre el reporte de balance general.

![Balance general](/img/screenshots/current/es/balance-sheet.cy.ts/balance-sheet.png)

**Paso 2.** Cambia moneda (USD/BTC).

![Balance general BTC](/img/screenshots/current/es/balance-sheet.cy.ts/balance-sheet-btc-currency.png)

**Paso 3.** Filtra por capa de balance (ejemplo: pendiente).

![Balance general capa pendiente](/img/screenshots/current/es/balance-sheet.cy.ts/balance-sheet-pending.png)

## Recorrido en Panel de Administración: Estado de Resultados

**Paso 1.** Abre el reporte de estado de resultados.

![Estado de resultados](/img/screenshots/current/es/profit-and-loss.cy.ts/profit-and-loss.png)

**Paso 2.** Cambia moneda de visualización.

![Estado de resultados BTC](/img/screenshots/current/es/profit-and-loss.cy.ts/profit-and-loss-btc-currency.png)

**Paso 3.** Filtra por capa (ejemplo: pendiente).

![Estado de resultados capa pendiente](/img/screenshots/current/es/profit-and-loss.cy.ts/profit-and-loss-pending.png)

