---
id: disbursal
title: Desembolso
sidebar_position: 3
---

# Desembolso

Los desembolsos son los **montos de principal** enviados al cliente.
Cada desembolso registra el monto liberado, lo vincula a la facilidad y emite eventos que crean nuevas obligaciones.
Esas obligaciones rastrean el principal (y cualquier comisión) que debe ser pagado según los términos de la facilidad.

## Precondiciones y validaciones

Antes de iniciar un desembolso, el dominio aplica controles estrictos:

- La facilidad debe estar en estado `Activo`.
- La fecha de desembolso debe ser anterior al vencimiento de la facilidad.
- Deben cumplirse requisitos de verificación del cliente (si la política los exige).
- La política de desembolso debe permitir un nuevo desembolso
  (comportamiento `SingleDisbursal` vs `MultipleDisbursal`).
- El CVL posterior al desembolso debe mantenerse en o por encima de `margin_call_cvl`.

Estos controles evitan crear otorgamientos fuera de política o con cobertura insuficiente.

## Modelo de estado y resultado

Los operadores suelen ver estas transiciones:

- `New`: desembolso inicializado y esperando decisión de gobernanza.
- `Approved`: se alcanzó el umbral de aprobación de gobernanza.
- `Confirmed`: desembolso liquidado; fondos acreditados y obligación creada.
- `Denied`: gobernanza rechaza; desembolso cancelado/revertido.

En términos prácticos, solo `Confirmed` significa que los fondos fueron efectivamente liberados y
que el seguimiento de repago ya está activo.

## Relación con obligaciones e intereses

Un desembolso confirmado crea una obligación de principal. Esa obligación entra luego al ciclo de
intereses:

- procesos periódicos registran devengo de intereses,
- el interés puede crear obligaciones de tipo interés,
- los pagos del cliente se asignan contra obligaciones pendientes según reglas de asignación.

Para operación, confirmar el desembolso es el inicio del monitoreo de repago y riesgo, no el final
del proceso.

## Recorrido en Panel de Administración: Crear y aprobar un desembolso

Este flujo continúa desde una facilidad de crédito activa y muestra cómo crear y aprobar un desembolso.

**Paso 23.** Desde la página de la facilidad activa, haz clic en **Crear** y luego en **Desembolso**.

![Iniciar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/23_click_initiate_disbursal_button.png)

**Paso 24.** Ingresa el monto del desembolso.

![Ingresar monto de desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/24_enter_disbursal_amount.png)

**Paso 25.** Envía la solicitud de desembolso.

![Enviar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/25_submit_disbursal_request.png)

**Paso 26.** Confirma que eres redirigido a la página de detalle del desembolso.

![Página de desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/26_disbursal_page.png)

**Paso 27.** Haz clic en **Aprobar** para ejecutar la aprobación de gobernanza.

![Aprobar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/27_approve.png)

**Paso 28.** Verifica que el estado cambie a **Confirmado**.

![Desembolso confirmado](/img/screenshots/current/es/credit-facilities.cy.ts/28_verify_disbursal_status_confirmed.png)

**Paso 29.** Confirma que el desembolso aparece en la lista de desembolsos.

![Desembolso en lista](/img/screenshots/current/es/credit-facilities.cy.ts/29_disbursal_in_list.png)

## Qué verificar después del Paso 29

- El estado del desembolso es `Confirmed`.
- El desembolso aparece bajo la facilidad y cliente esperados.
- El historial de la facilidad refleja ejecución/liquidación.
- Las vistas de repago muestran el impacto de la nueva obligación principal.
