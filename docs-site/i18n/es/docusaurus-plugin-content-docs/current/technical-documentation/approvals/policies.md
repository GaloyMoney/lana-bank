---
id: policies
title: Políticas de Aprobación
sidebar_position: 3
---

# Políticas de Aprobación

Una política define las reglas de aprobación para un tipo específico de operación. Cada tipo de operación (propuestas de líneas de crédito, desembolsos, retiros) tiene exactamente una política. Las políticas controlan si las operaciones se aprueban automáticamente o requieren revisión del comité y, en ese caso, cuántas aprobaciones se necesitan.

## Estructura de la Política

Cada política contiene:

- **Tipo de Proceso**: La categoría de operación que esta política regula. Existe una restricción de unicidad: solo puede existir una política por tipo de proceso.
- **Reglas de Aprobación**: Ya sea `AutoApprove` (las operaciones se aprueban instantáneamente) o `CommitteeThreshold` (las operaciones requieren votos del comité). Consulte los detalles a continuación.

## Tipos de Proceso

Tres tipos de proceso se registran al inicio del sistema:

| Tipo de Proceso | Identificador | Utilizado Por |
|-------------|-----------|---------|
| **Propuesta de Línea de Crédito** | `credit-facility-proposal` | Módulo de crédito: cuando un cliente acepta una propuesta |
| **Desembolso** | `disbursal` | Módulo de crédito: cuando un operador crea un desembolso |
| **Retiro** | `withdraw` | Módulo de depósitos: cuando un operador inicia un retiro |

La inicialización de políticas es idempotente: si la política para un tipo de proceso ya existe, la política existente se devuelve sin cambios. Esto permite que los módulos registren sus políticas de forma segura en cada inicio sin crear duplicados.

## Reglas de Aprobación

### Aprobación Automática del Sistema (Predeterminada)

Cada política se crea con reglas `AutoApprove` por defecto. Bajo este modo, cualquier proceso de aprobación iniciado contra esta política concluye inmediatamente con un resultado aprobado. No se requiere revisión humana.

Esta es la configuración apropiada cuando:
- El tipo de operación es de bajo riesgo y no requiere supervisión.
- El banco está en configuración inicial y aún no ha configurado comités.
- Entornos de prueba o desarrollo donde la fricción de aprobación no es deseable.

### Umbral del Comité

Cuando un administrador asigna un comité y un umbral a una política, las reglas cambian de `AutoApprove` a `CommitteeThreshold`. Bajo este modo:

- Cada nuevo proceso de aprobación requiere votos del comité asignado.
- El umbral especifica el número mínimo de votos de aprobación necesarios de los miembros elegibles.
- Un solo voto de rechazo de cualquier miembro elegible rechaza inmediatamente el proceso.

**Reglas de validación para la asignación de umbral:**
- El umbral debe ser al menos 1 (cero no está permitido).
- El umbral no debe exceder el número actual de miembros en el comité.
- Si el comité tiene 0 miembros, no se puede asignar un umbral.

Cambiar las reglas de la política solo afecta los procesos de aprobación futuros. Cualquier proceso ya en curso continúa bajo las reglas con las que fue creado (las reglas se guardan en cada proceso al momento de la creación).

## Configuración de Políticas

### Estado Inicial

Después del despliegue, las tres políticas existen con reglas `AutoApprove`. Todas las operaciones se aprueban automáticamente.

### Asignación de un Comité

Para requerir aprobación manual para un tipo de operación:

1. Cree un comité (consulte [Configuración de Comités](committees)).
2. Añada al menos un miembro al comité.
3. Navegue a la política para el tipo de operación deseado.
4. Asigne el comité y especifique un umbral (el número de aprobaciones requeridas).

Después de la asignación, todas las nuevas operaciones de ese tipo requerirán aprobación del comité. Los procesos existentes en curso no se ven afectados.

### Cambio de las Reglas

Puede reasignar un comité diferente o cambiar el umbral en cualquier momento. Se aplican las mismas reglas de validación: el umbral debe estar entre 1 y el número de miembros del nuevo comité. También puede revertir una política a aprobación automática actualizando las reglas (aunque el panel de administración normalmente hace esto asignando una configuración diferente).

## Cómo se Aplican las Reglas a los Procesos

Cuando se inicia un nuevo proceso de aprobación, las reglas actuales de la política se **copian** (se crea una instantánea) en el proceso. Esto significa:

- Si modificas las reglas de una política mientras un proceso está activo, el proceso activo continúa con sus reglas originales.
- La instantánea de las reglas incluye el ID del comité y el umbral, no la lista de miembros. La lista de miembros se obtiene actualizada en cada votación, por lo que los cambios de membresía sí afectan a los procesos activos (consulta [Configuración de Comités](committees) para más detalles sobre cómo funciona esto).

## Ejemplos Prácticos

**Escenario: Retiros de bajo valor auto-aprobados, de alto valor aprobados manualmente**

Lana no admite enrutamiento basado en montos dentro de una sola política. Todos los retiros usan la misma política. Si necesitas aprobación diferenciada según el monto, la solución operativa es usar auto-aprobación y confiar en auditorías posteriores para valores bajos, o requerir aprobación del comité para todos los retiros y confiar en tiempos de respuesta rápidos del comité.

**Escenario: Diferentes comités para diferentes operaciones**

Puedes asignar diferentes comités a diferentes políticas. Por ejemplo:
- Propuestas de líneas de crédito: asignadas a un "Comité de Riesgo Crediticio" con umbral 2
- Desembolsos: asignados al mismo comité u otro diferente con umbral 1
- Retiros: asignados a un "Comité de Operaciones" con umbral 1

Esto le da al banco flexibilidad para dirigir diferentes tipos de operaciones a los tomadores de decisiones apropiados.

## Recorrido en Panel de Administración: Asignar Comité y Resolver Acciones

### 1) Asignar comité a política

**Paso 12.** Abre la página de políticas.

![Visitar página de políticas](/img/screenshots/current/es/governance.cy.ts/12_step-visit-policies-page.png)

**Paso 13.** Selecciona una política.

![Seleccionar política](/img/screenshots/current/es/governance.cy.ts/13_step-select-policy.png)

**Paso 14.** Asigna comité y umbral.

![Asignar comité a política](/img/screenshots/current/es/governance.cy.ts/14_step-assign-committee-to-policy.png)

**Paso 15.** Verifica éxito de asignación.

![Verificar comité asignado](/img/screenshots/current/es/governance.cy.ts/15_step-verify-committee-assigned.png)

### 2) Revisar acciones pendientes

**Paso 16.** Abre la cola de acciones.

![Página de acciones](/img/screenshots/current/es/governance.cy.ts/16_step-view-actions-page.png)

**Paso 17.** Confirma que la solicitud pendiente aparece.

![Retiro pendiente visible](/img/screenshots/current/es/governance.cy.ts/17_step-verify-pending-withdrawal.png)

### 3) Aprobar o rechazar proceso

**Paso 18.** Abre el detalle de la solicitud.

![Detalle retiro para aprobación](/img/screenshots/current/es/governance.cy.ts/18_step-visit-withdrawal-details.png)

**Paso 19.** Haz clic en **Aprobar**.

![Clic en aprobar](/img/screenshots/current/es/governance.cy.ts/19_step-click-approve-button.png)

**Paso 20.** Verifica éxito y transición de estado.

![Éxito de aprobación](/img/screenshots/current/es/governance.cy.ts/20_step-verify-approval-success.png)

**Paso 21.** Abre solicitud para flujo de rechazo.

![Visitar retiro para rechazo](/img/screenshots/current/es/governance.cy.ts/21_step-visit-withdrawal-for-denial.png)

**Paso 22.** Haz clic en **Rechazar** e ingresa motivo.

![Clic en rechazar](/img/screenshots/current/es/governance.cy.ts/22_step-click-deny-button.png)

**Paso 23.** Verifica éxito del rechazo y estado terminal.

![Éxito de rechazo](/img/screenshots/current/es/governance.cy.ts/23_step-verify-denial-success.png)
