---
id: index
title: Gestión de Clientes
sidebar_position: 1
---

# Gestión de Clientes

El sistema de gestión de clientes es la base de identidad para todas las operaciones financieras en Lana. Cada cuenta de depósito, línea de crédito y transacción financiera se vincula en última instancia a un registro de cliente. El sistema cubre el ciclo de vida completo del cliente, desde el registro inicial y la verificación KYC hasta la gestión continua de la relación.

## Componentes del Sistema

| Componente | Módulo | Propósito |
|------------|--------|-----------|
| Gestión de Clientes | core-customer | Persistencia, perfiles y documentos |
| Procesamiento KYC | core-applicant | Integración con Sumsub |
| Onboarding de Usuarios | user-onboarding | Aprovisionamiento en Keycloak |

## Ciclo de Vida del Cliente

Un cliente progresa a través de varios estados desde la creación hasta las operaciones activas:

mermaid
graph LR
    CREATE["Creado<br/>(KYC pendiente)"] --> KYC["Verificación<br/>KYC"]
    KYC --> PROV["Aprovisionamiento<br/>(Keycloak + cuenta de depósito)"]
    PROV --> ACTIVE["Cliente<br/>activo"]
    ACTIVE --> FROZEN["Congelado"]
    FROZEN --> ACTIVE
    ACTIVE --> CLOSED["Cerrado"]
    FROZEN --> CLOSED
```

1. **Creación**: Un operador crea el registro del cliente en el panel de administración con correo electrónico, ID de Telegram opcional y tipo de cliente. El cliente comienza con la verificación KYC en estado `Pendiente`.
2. **Verificación KYC**: El operador genera un enlace de verificación de Sumsub. El cliente completa la verificación de identidad a través de la interfaz de Sumsub. Sumsub notifica al sistema mediante webhook cuando concluye la verificación.
3. **Aprovisionamiento**: Cuando se aprueba el KYC, el sistema emite eventos que activan el aprovisionamiento posterior. Se crea una cuenta de usuario de Keycloak para que el cliente pueda autenticarse, se envía un correo electrónico de bienvenida con las credenciales y se crea una cuenta de depósito.
4. **Operaciones activas**: El cliente ahora puede acceder al portal de clientes, recibir depósitos y solicitar líneas de crédito.

## Actividad de la cuenta de depósito

La actividad de las cuentas de depósito se gestiona automáticamente mediante un proceso en segundo plano periódico. El sistema determina la última fecha de actividad de cada cuenta de depósito a partir de la transacción más reciente registrada en la cuenta, o utiliza la fecha de creación de la cuenta si aún no existen transacciones. Luego, aplica umbrales configurables para determinar si la cuenta debe considerarse activa, inactiva o suspendida.

| Estado | Condición | Efecto |
|--------|-----------|--------|
| **Activa** | Actividad durante el último año | La cuenta se muestra como recientemente activa |
| **Inactiva** | Sin actividad durante 1-10 años | La cuenta se muestra como inactiva para seguimiento del operador |
| **Suspendida** | Sin actividad durante más de 10 años | La cuenta se muestra como inactiva por mucho tiempo para seguimiento del operador |

Este estado pertenece a la cuenta de depósito, no al cliente. La actividad es independiente del `status` operativo de la cuenta de depósito, por lo que un estado de actividad inactivo o suspendido no bloquea por sí solo los depósitos o retiros.

## Estados del Cliente

| Estado | Descripción |
|--------|-------------|
| ACTIVE | El cliente puede realizar operaciones |
| INACTIVE | La cuenta está inactiva |
| SUSPENDED | La cuenta está suspendida |

## Cerrar un cliente

Un operador puede cerrar una cuenta de cliente a través del panel de administración. El cierre es una acción permanente e irreversible que requiere que se cumplan todas las siguientes condiciones previas:

- Todas las **líneas de crédito** deben estar en estado `Cerrado`
- Todas las **propuestas de líneas de crédito** deben estar en un estado terminal (`Denegado`, `Aprobado` o `DenegadoPorCliente`)
- No debe haber **líneas de crédito pendientes** en espera de colateralización
- Todas las **cuentas de depósito** deben estar cerradas
- No debe haber **retiros pendientes** en ninguna cuenta de depósito

Cuando se cierra un cliente, el sistema desactiva la cuenta de usuario de Keycloak asociada, impidiendo futuras autenticaciones en el portal del cliente.

## Componentes del sistema

| Componente | Módulo | Propósito |
|-----------|--------|---------|
| **Gestión de clientes** | core-customer | Entidad de cliente, perfiles y estado KYC |
| **Procesamiento KYC** | core-customer (kyc) | Integración con API de Sumsub, manejo de callbacks de webhook |
| **Almacenamiento de documentos** | core-document-storage | Carga de archivos, almacenamiento en la nube, generación de enlaces de descarga |
| **Incorporación de usuarios** | lana-user-onboarding | Aprovisionamiento de usuarios en Keycloak mediante eventos de creación de clientes |

## Integración con otros módulos

El registro de cliente es referenciado por prácticamente todos los demás módulos del sistema:

- **Depósitos**: Cada cliente tiene una cuenta de depósito (creada automáticamente después de la aprobación KYC). El tipo de cliente determina a qué conjunto de cuentas del libro mayor pertenece la cuenta de depósito.
- **Crédito**: Las propuestas de facilidades crediticias están vinculadas a un cliente. La verificación KYC puede ser requerida antes de que se permitan los desembolsos.
- **Contabilidad**: El tipo de cliente determina la ubicación en el plan de cuentas tanto para los pasivos de depósitos como para las cuentas por cobrar de crédito.
- **Gobernanza**: Los procesos de aprobación para retiros y operaciones de crédito referencian al cliente indirectamente a través de las entidades asociadas.

## Documentación relacionada

- [Proceso de incorporación](onboarding) - Flujo completo de incorporación con KYC Sumsub
- [Gestión de documentos](documents) - Manejo de documentos del cliente
