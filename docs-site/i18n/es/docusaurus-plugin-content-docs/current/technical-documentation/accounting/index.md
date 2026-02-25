---
id: index
title: Contabilidad
sidebar_position: 1
---

# Contabilidad

El módulo de contabilidad proporciona contabilidad de partida doble para todas las operaciones financieras en Lana. Está construido sobre el motor de libro mayor Cala, que garantiza que cada transacción mantenga la ecuación contable fundamental: Activos = Pasivos + Patrimonio. Todas las operaciones de negocio del sistema (por ejemplo, depósitos, retiros, desembolsos, devengo de intereses, pagos y reconocimiento de comisiones, etc) generan asientos contables que fluyen por este marco.

## Plan de Cuentas

El plan de cuentas es una estructura jerárquica en árbol que organiza todas las cuentas financieras del sistema. Cada nodo del árbol representa una cuenta individual o un grupo de cuentas, identificado por códigos separados por puntos (por ejemplo, `1` para Activos, `11` para Cuentas por Cobrar de Corto Plazo, `11.01` para una cuenta específica por cobrar de un cliente).

### Categorías de Cuentas

Las cuentas se organizan en categorías financieras estándar:

| Categoría | Balance Normal | Descripción |
|----------|----------------|-------------|
| **Activos** | Débito | Recursos propiedad del banco (efectivo, cuentas por cobrar, colateral) |
| **Pasivos** | Crédito | Obligaciones frente a terceros (depósitos de clientes, cuentas por pagar) |
| **Patrimonio** | Crédito | Participación de los dueños en el banco (ganancias retenidas, capital) |
| **Ingresos** | Crédito | Ingresos devengados (intereses, comisiones) |
| **Costo de Ingresos** | Débito | Costos directos asociados a la generación de ingresos |
| **Gastos** | Débito | Costos operativos (provisiones, gastos de operación) |

El tipo de balance normal determina cómo se calcula el balance efectivo. Para cuentas con balance normal débito (activos, gastos), el balance efectivo es débitos menos créditos. Para cuentas con balance normal crédito (pasivos, patrimonio, ingresos), el balance efectivo es créditos menos débitos.

### Jerarquía de Cuentas

El plan de cuentas usa una jerarquía padre-hijo donde los códigos de nivel superior representan categorías principales y los subcódigos representan grupos cada vez más específicos. Por ejemplo:

- **1** — Activos
  - **11** — Cuentas por cobrar de corto plazo
    - **11.01** — Cuenta por cobrar de cliente individual
  - **12** — Efectivo y equivalentes
- **2** — Pasivos
  - **21** — Depósitos de clientes
- **3** — Patrimonio
  - **31** — Ganancias retenidas
    - **31.01** — Ganancias retenidas (ganancia)
    - **31.02** — Ganancias retenidas (pérdida)

Cada cuenta en la jerarquía está respaldada por un conjunto de cuentas en Cala, lo que permite agregar saldos de todas las cuentas hijas al generar reportes para un nodo padre.

### Configuración Base de Contabilidad

La configuración base de contabilidad mapea categorías de cuentas (así como **ganancias retenidas**, que están anidadas bajo **Patrimonio**) a códigos específicos dentro del plan de cuentas. Es requerida para operaciones contables como el cierre mensual y el cierre de año fiscal, y habilita la conexión de módulos de producto con el plan de cuentas.

Se expresa como JSON, donde cada clave es una categoría de cuenta (o un objetivo de **ganancias retenidas**, uno para ingreso neto positivo y otro para ingreso neto negativo) y cada valor representa un código del plan de cuentas.

```json
{
  "assets_code": "1",
  "liabilities_code": "2",
  "equity_code": "3",
  "equity_retained_earnings_gain_code": "31.01",
  "equity_retained_earnings_loss_code": "31.02",
  "revenue_code": "4",
  "cost_of_revenue_code": "5",
  "expenses_code": "6"
}
```

Cualquier nodo raíz del plan que no esté representado por un par clave/valor en la configuración base se considera fuera de balance. Los conjuntos de cuentas fuera de balance suelen usarse para contingencias o para representar transacciones que entran o salen del sistema.

### Configuración Inicial

El módulo de contabilidad requiere dos archivos de configuración para operar:

1. Plan de Cuentas (CSV)
2. Configuración Base Contable (JSON)

Esto debe definirse antes del arranque inicial mediante archivos de configuración en disco. También puede cargarse manualmente usando la mutación GraphQL `chartOfAccountsCsvImport`.

### Configuración de Integración

La configuración de integración mapea tipos de cliente y tipos de producto a posiciones específicas en el plan de cuentas. Cuando se crea una nueva cuenta de depósito o facilidad de crédito, el sistema genera automáticamente las cuentas hijas necesarias bajo los nodos padre correctos, según el tipo de cliente.

Por ejemplo, la configuración del módulo de crédito define códigos padre para:
- Cuentas por cobrar desembolsadas de corto plazo por cliente individual
- Cuentas por cobrar de intereses
- Cuentas de ingreso por intereses
- Cuentas de ingreso por comisiones
- Cuentas de colateral

Del mismo modo, la configuración del módulo de depósitos define códigos padre para:
- Cuentas de pasivo por depósitos de clientes (por tipo de cliente)
- Cuentas ómnibus para movimientos de fondos

Esta creación automática de cuentas garantiza que cada operación de negocio tenga su ubicación contable correcta sin requerir mantenimiento manual del plan por cada cliente nuevo.

## Estados Financieros

El sistema genera tres estados financieros principales a partir del plan de cuentas:

### Balance de Comprobación

El balance de comprobación lista todas las cuentas de primer nivel (hijas directas de la raíz del plan) con sus saldos débito y crédito en un momento específico. Su propósito principal es de verificación: el total de débitos debe ser igual al total de créditos. Si no lo es, existe un error contable en alguna parte del sistema.

El balance de comprobación es lo primero que un operador debe revisar al investigar discrepancias contables. Proporciona una vista rápida para validar la consistencia interna del libro mayor.

### Balance General

El balance general presenta la posición financiera del banco en una fecha específica organizando las cuentas en tres secciones:

- **Activos**: lo que el banco posee (cuentas por cobrar a clientes, efectivo, colateral retenido)
- **Pasivos**: lo que el banco debe (depósitos de clientes, cuentas por pagar)
- **Patrimonio**: interés residual (ganancias retenidas, capital aportado)

La ecuación Activos = Pasivos + Patrimonio debe cumplirse siempre. El balance general se construye agregando todas las cuentas bajo los códigos padre configurados para activos, pasivos y patrimonio.

### Estado de Resultados

El estado de resultados (P&L) muestra el desempeño financiero del banco en un período al calcular el ingreso neto:

- **Ingresos**: ingresos devengados durante el período (intereses de facilidades de crédito, comisiones de estructuración)
- **Costo de Ingresos**: costos directos asociados con la generación de ingresos
- **Gastos**: gastos operativos, provisiones por pérdidas crediticias y otros costos

Ingreso Neto = Ingresos - Costo de Ingresos - Gastos. Esta cifra representa la ganancia o pérdida del banco en el período de reporte. Al final de cada año fiscal, el ingreso neto se transfiere a ganancias retenidas en el balance general mediante el proceso de cierre.

## Modelo Operativo

Las páginas de contabilidad en el panel de administración exponen vistas estructuradas del libro mayor sobre contabilidad de partida doble de Cala. Normalmente los operadores usan:
- **Plan de Cuentas** para inspeccionar jerarquía y actividad por cuenta.
- **Balance de Comprobación** para validar consistencia de saldos.
- **Balance General** para posición financiera (activos, pasivos y patrimonio).
- **Estado de Resultados** para desempeño del período.

### Transacciones Manuales

Además de los asientos automáticos generados por operaciones de negocio, el sistema permite transacciones manuales para ajustes que no se originan en procesos automáticos. Son útiles para:

- Corregir errores detectados durante conciliaciones
- Registrar transacciones fuera del sistema
- Aplicar ajustes de cierre (por ejemplo, provisiones por pérdidas crediticias)

Las transacciones manuales siguen las mismas reglas de partida doble que las automáticas y quedan completamente auditadas.

## Recorrido en Panel de Administración: Configuración de Módulos

La configuración de módulos mapea flujos operativos (depósitos y crédito) a códigos padre del plan de cuentas. Estos mapeos son críticos porque determinan dónde se contabilizan las transacciones.

**Paso 1.** Abre la configuración de módulos.

![Configuración módulos](/img/screenshots/current/es/modules.cy.ts/1_modules_configuration.png)

**Paso 2.** Configura los mapeos contables de depósitos.

![Configuración depósitos](/img/screenshots/current/es/modules.cy.ts/2_deposit_configuration.png)

**Paso 3.** Configura los mapeos contables de crédito.

![Configuración crédito](/img/screenshots/current/es/modules.cy.ts/3_credit_configuration.png)

## Recorrido en Panel de Administración: Plan de Cuentas

**Paso 1.** Abre el plan de cuentas y valida la vista jerárquica.

![Vista plan de cuentas](/img/screenshots/current/es/chart-of-accounts.cy.ts/2_chart_of_account_view.png)

**Paso 2.** Abre el detalle de una cuenta de mayor para revisar movimientos.

![Detalle cuenta de mayor](/img/screenshots/current/es/chart-of-accounts.cy.ts/3_ledger_account_details.png)
