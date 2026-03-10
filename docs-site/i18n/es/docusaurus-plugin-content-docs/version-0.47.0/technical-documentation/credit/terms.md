---
id: terms
title: Términos
sidebar_position: 6
---

# Términos

Términos es un **objeto de valor** que captura los parámetros bajo los cuales opera una facilidad de crédito.
Se copia en una facilidad cuando la facilidad es creada y no cambia después.

## Campos

La estructura `ValoresDeTérminos` contiene los siguientes campos:

- `tasa_anual` – tasa de interés cobrada sobre el principal pendiente.
- `duración` – longitud total de la facilidad.
- `duración_vencimiento_intereses_desde_devengo` – tiempo desde el devengo de intereses hasta cuando ese interés vence.
- `duración_obligación_vencida_desde_vencimiento` – período de gracia opcional antes de que una obligación vencida se considere en mora.
- `duración_liquidación_obligación_desde_vencimiento` – buffer opcional antes de que una obligación en mora sea elegible para liquidación.
- `intervalo_ciclo_devengo` – cadencia con la que se generan nuevas obligaciones de interés.
- `intervalo_devengo` – frecuencia utilizada para calcular el interés devengado dentro de un ciclo.
- `tasa_comisión_única` – porcentaje de comisión tomado en el desembolso.
- `cvl_liquidación` – límite de valor del colateral que activa la liquidación.
- `cvl_llamada_margen` – límite de valor del colateral que activa una llamada de margen.
- `cvl_inicial` – límite de valor del colateral requerido en la creación de la facilidad.

## Plantillas de Términos

`PlantillaDeTérminos` es una entidad utilizada para persistir un conjunto reutilizable de valores de términos.
Las facilidades de crédito **no** están vinculadas a plantillas; en su lugar, los valores de una plantilla son
copiados en la facilidad en el momento de la creación.

## Importancia Operativa

Desde operación, las plantillas de términos funcionan como controles de riesgo:
- `tasa_anual` y duración definen costo y horizonte de obligaciones.
- `cvl_inicial`, `cvl_llamada_margen` y `cvl_liquidación` definen bordes de seguridad del colateral.
- `tasa_comisión_única` controla el cargo al desembolsar.
- política de desembolso define si el crédito es único o múltiple.

La calidad de la plantilla impacta aprobación, activación y monitoreo de riesgo posterior.

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
