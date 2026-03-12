---
id: terms
title: Términos
sidebar_position: 6
---

# Términos

Términos es un objeto de valor que captura el conjunto completo de parámetros bajo los cuales opera una línea de crédito. Cuando se crea una línea a partir de una propuesta, los términos se copian de una plantilla y se convierten en el contrato permanente que rige el comportamiento de la línea. No pueden modificarse después de la creación de la línea.

## Campos

### Tasa de Interés

**`annual_rate`** — La tasa de interés anualizada que se cobra sobre el principal pendiente, expresada como porcentaje.

El sistema utiliza esta tasa para calcular el interés diario: `principal * días * tasa / 365`, redondeado a centavos completos de USD. Este cálculo se ejecuta en cada período de acumulación (típicamente diario), y los resultados se acumulan a lo largo de cada ciclo de acumulación (típicamente mensual) antes de registrarse como una obligación de interés.

### Duración y Vencimiento

**`duration`** — La vida útil total de la línea de crédito, especificada en meses.

La fecha de vencimiento se calcula sumando esta cantidad de meses a la fecha de activación de la línea. La duración también determina la clasificación contable: las líneas con una duración de 12 meses o menos se clasifican como "corto plazo" y las líneas de más de 12 meses como "largo plazo". Esta clasificación determina qué conjunto de cuentas del libro mayor (cuentas por cobrar a corto plazo vs. largo plazo) se utiliza para los asientos contables de la línea.

Después del vencimiento:
- No se permiten nuevos desembolsos.
- No se inician nuevos ciclos de acumulación de intereses.
- Cualquier interés acumulado pero aún no registrado se consolida inmediatamente en una obligación final.
- La línea permanece abierta hasta que todas las obligaciones pendientes estén completamente pagadas.

### Calendario de Intereses

**`accrual_interval`** — La frecuencia con la que se calcula el interés dentro de cada ciclo. Típicamente se establece en `EndOfDay`, lo que significa que el interés se calcula diariamente sobre el saldo pendiente.

**`accrual_cycle_interval`** — La cadencia con la que el interés acumulado se convierte en una obligación. Típicamente se establece en `EndOfMonth`, lo que significa que los cálculos de interés diarios se suman y se registran como una obligación de interés pagadero al final de cada mes.

Estos dos intervalos crean un sistema de calendario de dos niveles. El `accrual_interval` controla la granularidad del cálculo de interés (diario es más preciso), mientras que el `accrual_cycle_interval` controla con qué frecuencia el cliente recibe una factura de interés. Esta separación permite al banco calcular el interés con granularidad fina mientras factura en intervalos prácticos.

### Cronología de las Obligaciones

**`interest_due_duration_from_accrual`** — El tiempo entre el momento en que se devenga el interés (fin del ciclo) y el momento en que la obligación resultante vence. Con un valor de `Days(0)`, la obligación de interés vence inmediatamente cuando termina el ciclo.

**`obligation_overdue_duration_from_due`** — Período de gracia opcional después de la fecha de vencimiento antes de que una obligación pase al estado "vencida". Cuando está configurado, las obligaciones pasan de `Due` a `Overdue` después de esta cantidad de días desde la fecha de vencimiento. Cuando no está configurado, las obligaciones nunca entran al estado vencido.

**`obligation_liquidation_duration_from_due`** — Período opcional después de la fecha de vencimiento en el cual una obligación impaga entra al estado "en mora" y se vuelve elegible para el proceso de cobranza/liquidación. Cuando no está configurado, las obligaciones nunca entran al estado en mora solo por el paso del tiempo.

Estos tres campos juntos crean la ruta de escalamiento gradual para las obligaciones impagas:

```
Devengado → Vencida → (período de gracia) → Vencida → (período de mora) → En Mora
```

### Comisión

**`one_time_fee_rate`** — Una comisión de estructuración/originación expresada como porcentaje del monto de la línea de crédito, cobrada al momento del desembolso. Si es cero, no se aplica ninguna comisión.

La comisión se calcula como `facility_amount * (rate / 100)`. Para líneas de crédito con un solo desembolso, la comisión se cobra como parte del desembolso inicial. Para líneas de crédito con múltiples desembolsos, se crea automáticamente un desembolso inicial que cubre únicamente el monto de la comisión al momento de la activación.

### Umbrales de Garantía

Tres umbrales porcentuales de garantía-a-valor (CVL) crean un sistema de seguridad gradual para gestionar la volatilidad de la garantía en Bitcoin:

**`initial_cvl`** — El nivel de garantía requerido para activar una línea de crédito y el nivel objetivo para la recuperación post-liquidación. Un valor más alto significa que el banco requiere un mayor margen de garantía antes de otorgar crédito.

**`margin_call_cvl`** — El umbral por debajo del cual la línea de crédito entra en estado de llamada de margen. También se usa como barrera para nuevos desembolsos: no se permite ningún desembolso si este llevaría el CVL por debajo de este nivel.

**`liquidation_cvl`** — El umbral más bajo. Cuando el CVL cae por debajo de este nivel, se inicia automáticamente una liquidación parcial para vender suficiente garantía y restaurar el CVL por encima del nivel inicial.

**Validación**: Los tres umbrales deben estar estrictamente ordenados: `initial_cvl > margin_call_cvl > liquidation_cvl`. La igualdad en cualquier límite se rechaza al momento de creación de la plantilla.

Los cuatro estados de garantía y sus implicaciones operativas:

| Posición CVL | Estado | Efecto |
|-------------|-------|--------|
| Por encima de `initial_cvl` | Totalmente Garantizado | Operaciones normales, desembolsos permitidos |
| Entre `margin_call_cvl` e `initial_cvl` | Totalmente Garantizado | Operaciones normales, pero desembolsos bloqueados si llevarían el CVL por debajo de la llamada de margen |
| Entre `liquidation_cvl` y `margin_call_cvl` | Bajo Llamada de Margen | Se notifica al prestatario para que deposite garantía adicional |
| Por debajo de `liquidation_cvl` | Bajo Liquidación | Liquidación parcial iniciada automáticamente |

Un búfer de histéresis previene la oscilación rápida entre estados cuando el CVL fluctúa cerca del límite de un umbral.

### Política de Desembolso

**`disbursal_policy`** — Controla si el monto de la línea de crédito se retira en su totalidad de una vez o de forma incremental.

- **Desembolso Único**: El monto total de la línea de crédito se desembolsa automáticamente al momento de la activación como un desembolso preaprobado. No son posibles desembolsos adicionales.
- **Desembolso Múltiple**: Al momento de la activación, solo se desembolsa la comisión de estructuración (si aplica). El cliente puede solicitar desembolsos adicionales a lo largo del tiempo, cada uno requiriendo su propia aprobación de gobernanza. Esto es útil para líneas de capital de trabajo donde las necesidades de efectivo del prestatario varían.

## Plantillas de Términos

Una `TermsTemplate` es una colección reutilizable y nombrada de valores de términos. Las plantillas sirven como definiciones de productos: el banco crea plantillas para diferentes productos de préstamo (por ejemplo, "Préstamo Garantizado Estándar a 12 Meses", "Línea de Capital de Trabajo") y los operadores seleccionan una plantilla al crear una propuesta.

Características principales:

- **Copiadas, no vinculadas**: Cuando se crea una propuesta a partir de una plantilla, los valores de los términos se copian en la propuesta. Actualizar una plantilla posteriormente no cambia las líneas de crédito existentes.
- **Nombres únicos**: Cada plantilla debe tener un nombre único.
- **Actualizables**: Los valores de las plantillas pueden modificarse en cualquier momento. Solo las propuestas futuras que utilicen la plantilla se verán afectadas.
- **Controles de riesgo**: Las plantillas son efectivamente controles de riesgo, no solo configuración. Los umbrales de CVL, las tasas de interés y las tasas de comisiones definidas en una plantilla determinan directamente los límites de seguridad y la economía de cada línea de crédito creada a partir de ella.

## Importancia Operativa

Desde una perspectiva operativa, las plantillas de términos son la configuración más impactante del sistema:

- **`annual_rate`** y **`duration`** configuran el costo del préstamo y el cronograma de la obligación.
- **`initial_cvl`**, **`margin_call_cvl`** y **`liquidation_cvl`** definen los límites de seguridad del colateral que protegen al banco contra la volatilidad del precio de BTC.
- **`one_time_fee_rate`** controla los ingresos por comisiones iniciales.
- **`accrual_cycle_interval`** determina la frecuencia de facturación (la facturación mensual es estándar).
- **`disbursal_policy`** controla si el préstamo es de desembolso único o incremental.

La calidad de las plantillas impacta directamente el comportamiento de aprobación, los requisitos de activación, los patrones de devengo de intereses y el monitoreo de colateral subsiguiente. Umbrales o intervalos incorrectos en las plantillas de términos producen un comportamiento de ciclo de vida incorrecto para cada línea de crédito creada a partir de ellas.

## Recorrido en Panel de Administración: Crear y Actualizar Plantilla de Términos

### A) Crear plantilla

**Paso 1.** Abre la página de plantillas de términos.

![Visitar plantillas de términos](/img/screenshots/current/es/terms-templates.cy.ts/1_visit_terms_templates_page.png)

**Paso 2.** Haz clic en **Crear**.

![Clic crear plantilla](/img/screenshots/current/es/terms-templates.cy.ts/2_click_create_button.png)

**Paso 3.** Ingresa nombre único.

![Ingresar nombre plantilla](/img/screenshots/current/es/terms-templates.cy.ts/3_enter_template_name.png)

**Paso 4.** Ingresa tasa anual.

![Ingresar tasa anual](/img/screenshots/current/es/terms-templates.cy.ts/4_enter_annual_rate.png)

**Paso 5.** Ingresa unidades de duración.

![Ingresar duración](/img/screenshots/current/es/terms-templates.cy.ts/5_enter_duration_units.png)

**Paso 6.** Ingresa `cvl_inicial`.

![Ingresar cvl inicial](/img/screenshots/current/es/terms-templates.cy.ts/6_enter_initial_cvl.png)

**Paso 7.** Ingresa `cvl_llamada_margen`.

![Ingresar cvl llamada margen](/img/screenshots/current/es/terms-templates.cy.ts/7_enter_margin_call_cvl.png)

**Paso 8.** Ingresa `cvl_liquidación`.

![Ingresar cvl liquidación](/img/screenshots/current/es/terms-templates.cy.ts/8_enter_liquidation_cvl.png)

**Paso 9.** Ingresa tasa de comisión única.

![Ingresar tasa comisión](/img/screenshots/current/es/terms-templates.cy.ts/9_enter_fee_rate.png)

**Paso 10.** Selecciona política de desembolso.

![Seleccionar política desembolso](/img/screenshots/current/es/terms-templates.cy.ts/10_select_disbursal_policy.png)

**Paso 11.** Envía plantilla.

![Enviar plantilla términos](/img/screenshots/current/es/terms-templates.cy.ts/11_submit_terms_template.png)

**Paso 12.** Verifica detalle y URL.

![Verificar creación plantilla](/img/screenshots/current/es/terms-templates.cy.ts/12_verify_terms_template_creation.png)

**Paso 13.** Verifica presencia en lista.

![Plantilla en lista](/img/screenshots/current/es/terms-templates.cy.ts/13_terms_template_in_list.png)

### B) Actualizar plantilla

**Paso 14.** Abre detalle de plantilla.

![Detalle plantilla](/img/screenshots/current/es/terms-templates.cy.ts/14_terms_template_details.png)

**Paso 15.** Haz clic en **Actualizar**.

![Clic actualizar plantilla](/img/screenshots/current/es/terms-templates.cy.ts/15_click_update_button.png)

**Paso 16.** Modifica campo(s) necesarios (ejemplo: tasa anual).

![Actualizar tasa anual](/img/screenshots/current/es/terms-templates.cy.ts/16_update_annual_rate.png)

**Paso 17.** Envía cambios.

![Enviar actualización plantilla](/img/screenshots/current/es/terms-templates.cy.ts/17_submit_update.png)

**Paso 18.** Verifica mensaje de éxito.

![Éxito actualización plantilla](/img/screenshots/current/es/terms-templates.cy.ts/18_update_success.png)
