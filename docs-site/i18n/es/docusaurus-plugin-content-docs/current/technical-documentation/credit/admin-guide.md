---
id: admin-guide
title: "Guía del Panel de Administración: Facilidades de Crédito"
sidebar_position: 8
---

# Recorrido de Facilidades de Crédito

Esta guía recorre el ciclo de vida completo de una facilidad de crédito usando el panel de
administración — desde la creación de una propuesta hasta el desembolso de fondos. Cada paso
incluye una captura de pantalla que muestra exactamente lo que verás en la interfaz.

Una facilidad de crédito es un acuerdo de préstamo legalmente vinculante entre el banco y un
cliente. Establece un límite de crédito máximo, especifica los términos del préstamo (tasas de
interés, comisiones, parámetros de riesgo) y define los requisitos de garantía que deben
cumplirse antes de que los fondos puedan desembolsarse.

---

## 1. Crear una Propuesta de Facilidad de Crédito

Una facilidad de crédito comienza como una **propuesta** creada por un operador bancario en
nombre de un cliente. La propuesta especifica el monto de la facilidad y la vincula a una
**Plantilla de Términos** que define las tasas de interés, los requisitos de garantía y otros
parámetros.

**Paso 1.** Desde la página de un cliente, haz clic en el botón **Crear**. Aparecerá un menú
desplegable con las acciones disponibles.

![Hacer clic en Crear](/img/screenshots/current/es/credit-facilities.cy.ts/01_click_create_proposal_button.png)

**Paso 2.** Selecciona **Facilidad de Crédito** para abrir el formulario de creación de
propuesta.

![Abrir formulario de propuesta](/img/screenshots/current/es/credit-facilities.cy.ts/02_open_proposal_form.png)

**Paso 3.** Ingresa el monto deseado de la facilidad y selecciona una Plantilla de Términos
del menú desplegable. La plantilla de términos define la tasa de interés, los umbrales de CVL,
la duración y la estructura de comisiones.

![Ingresar monto de facilidad](/img/screenshots/current/es/credit-facilities.cy.ts/03_enter_facility_amount.png)

**Paso 4.** Haz clic en **Crear** para enviar la propuesta.

![Enviar formulario de propuesta](/img/screenshots/current/es/credit-facilities.cy.ts/04_submit_proposal_form.png)

**Paso 5.** La propuesta se creó exitosamente. Serás redirigido a la página de detalles de la
propuesta donde puedes revisar todos los detalles. El estado inicial es **Pendiente de
Aprobación del Cliente**.

![Propuesta creada exitosamente](/img/screenshots/current/es/credit-facilities.cy.ts/05_proposal_created_success.png)

**Paso 6.** Verifica que la propuesta aparece en la lista de propuestas de facilidades de
crédito.

![Propuesta en lista](/img/screenshots/current/es/credit-facilities.cy.ts/06_proposal_in_list.png)

---

## 2. Aceptación del Cliente y Aprobación Interna

Antes de que una facilidad pueda continuar, el cliente debe aceptar los términos de la
propuesta. Después de la aceptación del cliente, la propuesta pasa por un proceso de aprobación
interna gestionado por el módulo de gobernanza, donde los miembros del comité revisan y votan
sobre la propuesta.

### Aceptación del Cliente

**Paso 7.** Navega a la página de detalles de la propuesta.

![Visitar página de propuesta](/img/screenshots/current/es/credit-facilities.cy.ts/07_visit_proposal_page.png)

**Paso 8.** Haz clic en el botón **El Cliente Acepta** para registrar la aceptación del
cliente de los términos de la propuesta.

![Botón de aprobación del cliente](/img/screenshots/current/es/credit-facilities.cy.ts/08_customer_approval_button.png)

**Paso 9.** Confirma la aceptación del cliente en el diálogo.

![Diálogo de aprobación del cliente](/img/screenshots/current/es/credit-facilities.cy.ts/09_customer_approval_dialog.png)

**Paso 10.** El estado de la propuesta cambia a **Pendiente de Aprobación**, indicando que
ahora requiere aprobación interna del comité.

![Propuesta pendiente de aprobación](/img/screenshots/current/es/credit-facilities.cy.ts/10_proposal_pending_approval_status.png)

### Aprobación Interna

**Paso 11.** Haz clic en el botón **Aprobar** para iniciar el proceso de aprobación interna.

![Botón de aprobar propuesta](/img/screenshots/current/es/credit-facilities.cy.ts/11_approve_proposal_button.png)

**Paso 12.** Confirma la aprobación en el diálogo. Dependiendo de la política de gobernanza,
varios miembros del comité pueden necesitar aprobar antes de que la propuesta sea completamente
aprobada.

![Diálogo de aprobar propuesta](/img/screenshots/current/es/credit-facilities.cy.ts/12_approve_proposal_dialog.png)

**Paso 13.** El estado de la propuesta cambia a **Aprobado**.

![Propuesta aprobada](/img/screenshots/current/es/credit-facilities.cy.ts/13_proposal_approved_status.png)

**Paso 14.** Haz clic en **Ver Facilidad Pendiente** para navegar a la facilidad de crédito
pendiente recién creada.

![Botón ver facilidad pendiente](/img/screenshots/current/es/credit-facilities.cy.ts/14_view_pending_facility_button.png)

---

## 3. Colateralización y Activación

Después de la aprobación, la propuesta se convierte en una **Facilidad de Crédito Pendiente**.
El cliente debe depositar garantía en Bitcoin que cumpla con la proporción de Valor de Garantía
sobre Préstamo (CVL) definida en los términos de la facilidad. La facilidad se activa
automáticamente una vez que se alcanza el umbral de garantía.

**Paso 15.** La página de la facilidad pendiente muestra el estado actual como **Pendiente de
Colateralización** y muestra los requisitos de garantía.

![Estado inicial de facilidad pendiente](/img/screenshots/current/es/credit-facilities.cy.ts/15_pending_facility_initial_state.png)

**Paso 16.** Haz clic en el botón **Actualizar Garantía** para registrar un depósito de
garantía.

![Hacer clic en actualizar garantía](/img/screenshots/current/es/credit-facilities.cy.ts/16_click_update_collateral_button.png)

**Paso 17.** Ingresa el nuevo monto de garantía. La página muestra el monto objetivo necesario
para cumplir con el requisito de CVL inicial.

![Ingresar valor de garantía](/img/screenshots/current/es/credit-facilities.cy.ts/17_enter_new_collateral_value.png)

**Paso 18.** La garantía se actualiza exitosamente.

![Garantía actualizada](/img/screenshots/current/es/credit-facilities.cy.ts/18_collateral_updated.png)

**Paso 19.** El estado de la facilidad pendiente cambia a **Completado**, indicando que los
requisitos de garantía se han cumplido y la facilidad ha sido activada.

![Facilidad pendiente completada](/img/screenshots/current/es/credit-facilities.cy.ts/19_pending_facility_completed.png)

**Paso 20.** Haz clic en **Ver Facilidad** para navegar a la facilidad de crédito ahora activa.

![Botón ver facilidad](/img/screenshots/current/es/credit-facilities.cy.ts/20_view_facility_button.png)

**Paso 21.** Verifica que el estado de la facilidad de crédito es **Activo**. En este punto,
el devengo de intereses comienza y el cliente puede solicitar desembolsos.

![Verificar estado activo](/img/screenshots/current/es/credit-facilities.cy.ts/21_verify_active_status.png)

**Paso 22.** La facilidad también aparece en la lista de facilidades de crédito.

![Facilidad de crédito en lista](/img/screenshots/current/es/credit-facilities.cy.ts/22_credit_facility_in_list.png)

---

## 4. Desembolso

Con una facilidad de crédito activa, el cliente puede recibir **desembolsos** — montos de
principal enviados al cliente desde la facilidad. Cada desembolso pasa por su propio proceso
de aprobación y crea obligaciones que rastrean el cronograma de pago.

**Paso 23.** Desde la página de la facilidad de crédito activa, haz clic en **Crear** y
luego **Desembolso** para iniciar un nuevo desembolso.

![Iniciar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/23_click_initiate_disbursal_button.png)

**Paso 24.** Ingresa el monto del desembolso. El monto debe estar dentro del límite de crédito
disponible de la facilidad.

![Ingresar monto de desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/24_enter_disbursal_amount.png)

**Paso 25.** Envía la solicitud de desembolso.

![Enviar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/25_submit_disbursal_request.png)

**Paso 26.** El desembolso se crea y serás redirigido a la página de detalles del desembolso.

![Página de desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/26_disbursal_page.png)

**Paso 27.** Haz clic en **Aprobar** para aprobar el desembolso. Al igual que las propuestas,
los desembolsos pueden requerir la aprobación de varios miembros del comité dependiendo de la
política de gobernanza.

![Aprobar desembolso](/img/screenshots/current/es/credit-facilities.cy.ts/27_approve.png)

**Paso 28.** El estado del desembolso cambia a **Confirmado**. Los fondos se han acreditado
en la cuenta de depósito del cliente y se ha creado una obligación correspondiente para
rastrear el pago.

![Desembolso confirmado](/img/screenshots/current/es/credit-facilities.cy.ts/28_verify_disbursal_status_confirmed.png)

**Paso 29.** El desembolso aparece en la lista de desembolsos.

![Desembolso en lista](/img/screenshots/current/es/credit-facilities.cy.ts/29_disbursal_in_list.png)
