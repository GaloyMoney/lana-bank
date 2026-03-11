---
id: index
title: Contabilidad
sidebar_position: 1
---

# Contabilidad

El módulo de contabilidad proporciona contabilidad por partida doble para todas las operaciones financieras en Lana. Está construido sobre el motor de libro mayor Cala, que garantiza que cada transacción mantenga la ecuación contable fundamental: Activos = Pasivos + Patrimonio. Todas las operaciones comerciales en el sistema — depósitos, retiros, desembolsos, devengo de intereses, pagos y reconocimiento de comisiones — finalmente producen asientos contables que fluyen a través de este marco contable.

## Plan de Cuentas

El plan de cuentas es una estructura jerárquica en forma de árbol que organiza todas las cuentas financieras en el sistema. Cada nodo en el árbol representa ya sea una cuenta individual o un grupo de cuentas, identificado por códigos de cuenta separados por puntos (por ejemplo, "1" para Activos, "11" para Cuentas por Cobrar a Corto Plazo, "11.01" para una cuenta por cobrar de un cliente específico).

### Categorías de Cuentas

Las cuentas están organizadas en las categorías financieras estándar:

| Categoría | Saldo Normal | Descripción |
|----------|---------------|-------------|
| **Activos** | Débito | Recursos propiedad del banco (efectivo, cuentas por cobrar, garantías) |
| **Pasivos** | Crédito | Obligaciones adeudadas a terceros (depósitos de clientes, cuentas por pagar) |
| **Patrimonio** | Crédito | Participación del propietario en el banco (utilidades retenidas, capital) |
| **Ingresos** | Crédito | Ganancias obtenidas (ingresos por intereses, ingresos por comisiones) |
| **Costo de Ingresos** | Débito | Costos directos asociados con la generación de ingresos |
| **Gastos** | Débito | Costos operativos (provisiones, gastos operacionales) |

El tipo de saldo normal determina cómo se calcula el saldo efectivo. Para cuentas con saldo normal deudor (activos, gastos), el saldo efectivo es débitos menos créditos. Para cuentas con saldo normal acreedor (pasivos, patrimonio, ingresos), el saldo efectivo es créditos menos débitos.

### Jerarquía de Cuentas

El plan de cuentas utiliza una jerarquía padre-hijo donde los códigos de nivel superior representan categorías principales y los subcódigos representan grupos de cuentas cada vez más específicos. Por ejemplo:

- **1** — Activos
  - **11** — Cuentas por Cobrar a Corto Plazo
    - **11.01** — Cuenta individual por cobrar del cliente
  - **12** — Efectivo y Equivalentes
- **2** — Pasivos
  - **21** — Depósitos de Clientes
- **3** — Patrimonio
  - **31** — Ganancias Retenidas
    - **31.01** — Ganancias Retenidas (Ganancia)
    - **31.02** — Ganancias Retenidas (Pérdida)

Cada cuenta en la jerarquía está respaldada por un conjunto de cuentas en el libro mayor de Cala, lo que permite al sistema agregar saldos de todas las cuentas secundarias al generar informes para un nodo padre.

### Configuración de Base Contable

La configuración de base contable mapea las categorías de cuentas (así como las **ganancias retenidas**, que están anidadas bajo la categoría **Patrimonio**) a códigos específicos en el plan de cuentas. Es requerida para operaciones contables como el cierre mensual y de fin de año fiscal y permite adjuntar módulos de productos al plan de cuentas.

Toma la forma de JSON, donde cada clave es una categoría de cuenta (o un destino de **ganancias retenidas**, uno para ingresos netos positivos y otro para ingresos netos negativos) y cada valor representa un código en el plan de cuentas.

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

Cualquier nodo de nivel raíz en el plan que no esté representado por un par clave/valor en la configuración de base contable se considera fuera de balance. Los conjuntos de cuentas fuera de balance se utilizan típicamente para rastrear contingencias o representar transacciones que entran o salen del sistema.

### Configuración

El módulo de contabilidad requiere dos archivos de configuración para funcionar:

1. Plan de Cuentas (CSV)
2. Configuración Base de Contabilidad (JSON)

Estos deben configurarse antes del arranque inicial mediante archivos de configuración en disco; sin embargo, la mutación GraphQL `chartOfAccountsCsvImport` expone la capacidad de realizar este paso manualmente.

### Configuración de Integración

La configuración de integración mapea los tipos de clientes y tipos de productos a posiciones específicas en el plan de cuentas. Cuando se crea una nueva cuenta de depósito o línea de crédito, el sistema genera automáticamente las cuentas hijas necesarias bajo los nodos padre correctos según el tipo de cliente.

Por ejemplo, la configuración del módulo de crédito especifica códigos padre para:
- Cuentas de préstamos por cobrar desembolsados individuales a corto plazo
- Cuentas de intereses por cobrar
- Cuentas de ingresos por intereses
- Cuentas de ingresos por comisiones
- Cuentas de garantías

De manera similar, la configuración del módulo de depósitos especifica códigos padre para:
- Cuentas de pasivo por depósitos de clientes (por tipo de cliente)
- Cuentas ómnibus para movimientos de fondos

Esta creación automática de cuentas garantiza que cada operación comercial tenga un lugar contable apropiado sin requerir mantenimiento manual del plan de cuentas para cada nuevo cliente.

## Estados Financieros

El sistema genera tres estados financieros principales a partir del plan de cuentas:

### Balance de Comprobación

El balance de comprobación enumera todas las cuentas de primer nivel (hijas directas de la raíz del plan) con sus saldos deudores y acreedores en un momento específico. Su propósito principal es la verificación: el total de débitos debe ser igual al total de créditos. Si no coinciden, existe un error contable en alguna parte del sistema.

El balance de comprobación es lo primero que un operador debe revisar al investigar discrepancias contables. Proporciona una vista rápida de si el libro mayor es internamente consistente.

### Balance General

El balance general presenta la posición financiera del banco en una fecha específica organizando las cuentas en tres secciones:

- **Activos**: Lo que el banco posee (cuentas por cobrar a clientes, efectivo, garantías retenidas)
- **Pasivos**: Lo que el banco adeuda (depósitos de clientes, cuentas por pagar)
- **Patrimonio**: El interés residual (utilidades retenidas, capital aportado)

La ecuación fundamental Activos = Pasivos + Patrimonio debe cumplirse siempre. El balance general se construye agregando todas las cuentas bajo los códigos padre de activos, pasivos y patrimonio configurados.

### Estado de Resultados

El estado de resultados (también conocido como estado de pérdidas y ganancias) muestra el desempeño financiero del banco durante un período mediante el cálculo del ingreso neto:

- **Ingresos**: Ingresos obtenidos durante el período (ingresos por intereses de líneas de crédito, ingresos por comisiones de estructuración)
- **Costo de Ingresos**: Costos directos asociados con la generación de ingresos
- **Gastos**: Gastos operativos, provisiones para pérdidas crediticias y otros costos

Ingreso Neto = Ingresos - Costo de Ingresos - Gastos. Esta cifra representa la ganancia o pérdida del banco para el período reportado. Al final de cada año fiscal, el ingreso neto se transfiere a las utilidades retenidas en el balance general mediante el proceso de cierre.

## Modelo Operativo

Las páginas de contabilidad en el panel de administración exponen vistas estructuradas en libro mayor respaldadas por la contabilidad de partida doble de Cala. Los operadores suelen utilizar:
- **Plan de Cuentas** para inspeccionar la jerarquía y la actividad a nivel de cuenta.
- **Balance de Comprobación** para validar la consistencia del saldo del libro mayor.
- **Balance General** para la posición (activos, pasivos, patrimonio).
- **Estado de Resultados** para el desempeño del período.

### Transacciones Manuales

Además de los asientos contables automatizados generados por las operaciones comerciales, el sistema admite transacciones manuales para ajustes que no se originan en procesos automatizados. Estas son útiles para:

- Corregir errores descubiertos durante la conciliación
- Registrar transacciones fuera del sistema
- Realizar ajustes de fin de período (como provisiones para pérdidas crediticias)

Las transacciones manuales siguen las mismas reglas de partida doble que las automatizadas y están completamente auditadas.

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
