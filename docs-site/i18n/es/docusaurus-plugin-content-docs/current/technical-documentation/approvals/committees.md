---
id: committees
title: Comités de Aprobación
sidebar_position: 2
---

# Comités de Aprobación

Un comité es un grupo designado de usuarios autorizados que toman decisiones sobre procesos de aprobación. Los comités proporcionan el elemento humano en el sistema de gobernanza: cuando una política se configura con un umbral de comité, los miembros del comité son las personas que votan para aprobar o denegar operaciones.

## Estructura del Comité

Cada comité tiene:

- **Nombre**: Un identificador legible para humanos (por ejemplo, "Comité de Crédito", "Comité de Operaciones"). Debe ser único.
- **Miembros**: Un conjunto de usuarios que son elegibles para votar en los procesos de aprobación asignados a este comité.

Los comités no están vinculados a tipos de operaciones específicas. Un solo comité puede asignarse a múltiples políticas (por ejemplo, el mismo comité de crédito podría aprobar tanto propuestas de facilidades como desembolsos). Por el contrario, cada política solo puede hacer referencia a un comité a la vez.

## Gestión de Miembros

### Agregar Miembros

Los miembros del comité se identifican por su ID de usuario. Cuando se agrega un usuario a un comité, se vuelve elegible para votar en cualquier proceso de aprobación activo que haga referencia a ese comité. Agregar un miembro que ya está en el comité no tiene ningún efecto (la operación es idempotente).

### Eliminar Miembros

Cuando se elimina un miembro de un comité, ya no puede votar en nuevos procesos de aprobación. Sin embargo, cualquier voto que ya haya emitido en procesos existentes permanece válido. Eliminar a alguien que no es miembro no tiene ningún efecto.

### Impacto en Procesos Activos

Los cambios en la membresía pueden afectar los procesos de aprobación en curso:

- **Agregar un miembro** amplía el conjunto de votantes elegibles. El nuevo miembro puede votar inmediatamente en cualquier proceso activo que utilice ese comité.
- **Eliminar un miembro** reduce el conjunto de votantes elegibles. Si los miembros elegibles restantes ya no pueden alcanzar el umbral (por ejemplo, el umbral es 3 pero solo quedan 2 miembros elegibles, y menos de 3 ya han aprobado), el proceso se deniega automáticamente.

Esto se debe a que la lógica de aprobación verifica si todavía es matemáticamente posible alcanzar el umbral con el conjunto elegible actual. Si no lo es, el proceso concluye como denegado.

## Reglas de Votación

Cuando un miembro del comité vota en un proceso de aprobación:

1. **Cada miembro vota una vez**: Un miembro no puede cambiar su voto después de emitirlo. Intentar votar nuevamente (en cualquier dirección) es rechazado.
2. **La aprobación se acumula**: Los votos de aprobación se cuentan contra el umbral. Cuando el número de aprobaciones de miembros elegibles alcanza o supera el umbral, el proceso es aprobado.
3. **El rechazo es inmediato**: Un solo voto de rechazo de cualquier miembro elegible del comité niega inmediatamente todo el proceso, independientemente de cuántas aprobaciones ya se hayan emitido. Esto otorga a cada miembro del comité poder de veto efectivo.
4. **Los no miembros no pueden votar**: Solo los usuarios que son miembros actuales del comité asignado y que aún no han votado son elegibles para votar.

### Cálculo del Umbral

La verificación de aprobación funciona de la siguiente manera:

1. Obtener el conjunto de miembros actuales del comité (los votantes elegibles).
2. Intersectar los votantes elegibles con el conjunto de miembros que han votado a favor de la aprobación.
3. Si el recuento de la intersección cumple con el umbral, el proceso es aprobado.
4. Si algún miembro elegible ha rechazado, el proceso es denegado.
5. Si el número de miembros elegibles es menor que el umbral (imposible de aprobar), el proceso es denegado.
6. De lo contrario, el proceso permanece en curso, esperando más votos.

## Consideraciones Operativas

- **Crear comités antes de asignarlos a políticas**: Un comité debe existir y tener miembros antes de poder ser asignado significativamente a una política. Asignar un comité vacío a una política haría que cada proceso de aprobación fuera instantáneamente denegado (umbral inalcanzable).
- **El umbral no debe exceder el número de miembros**: Al asignar un comité a una política, el umbral se valida contra el número actual de miembros. Un umbral de 3 es rechazado si el comité tiene solo 2 miembros.
- **Tamaño del comité y disponibilidad**: En la práctica, los comités deben tener más miembros que el umbral requerido para tener en cuenta la no disponibilidad de los miembros. Un umbral de 2 con exactamente 2 miembros significa que ambos deben aprobar; un umbral de 2 con 4 miembros proporciona redundancia.

## Tutorial del Panel de Administración: Crear Comité y Agregar Miembros

### 1) Crear comité

**Paso 1.** Visitar la página de comités.

![Visitar la página de comités](/img/screenshots/current/en/governance.cy.ts/1_step-visit-committees.png)

**Paso 2.** Hacer clic en **Crear Comité**.

![Hacer clic en crear comité](/img/screenshots/current/en/governance.cy.ts/2_step-click-create-committee-button.png)

**Paso 3.** Ingresar el nombre del comité.

![Completar nombre del comité](/img/screenshots/current/en/governance.cy.ts/3_step-fill-committee-name.png)

**Paso 4.** Enviar la creación del comité.

![Enviar creación del comité](/img/screenshots/current/en/governance.cy.ts/4_step-submit-committee-creation.png)

**Paso 5.** Verificar el éxito.

![Comité creado exitosamente](/img/screenshots/current/en/governance.cy.ts/5_step-committee-created-successfully.png)

**Paso 6.** Confirmar que el comité aparece en la lista.

![Lista de comités](/img/screenshots/current/en/governance.cy.ts/6_step-view-committees-list.png)

### 2) Agregar miembro

**Paso 7.** Abrir los detalles del comité.

![Detalles del comité](/img/screenshots/current/en/governance.cy.ts/7_step-visit-committee-details.png)

**Paso 8.** Hacer clic en **Agregar Miembro**.

![Botón agregar miembro](/img/screenshots/current/en/governance.cy.ts/8_step-click-add-member-button.png)

**Paso 9.** Seleccionar la asignación de rol/miembro.

![Seleccionar rol de administrador](/img/screenshots/current/en/governance.cy.ts/9_step-select-admin-role.png)

**Paso 10.** Enviar la adición del miembro.

![Enviar agregar miembro](/img/screenshots/current/en/governance.cy.ts/10_step-submit-add-member.png)

**Paso 11.** Verificar que el miembro se haya agregado exitosamente.

![Verificar miembro agregado](/img/screenshots/current/en/governance.cy.ts/11_step-verify-member-added.png)
