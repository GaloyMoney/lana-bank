---
id: functional-architecture
title: Arquitectura Funcional
sidebar_label: Arquitectura Funcional
sidebar_position: 1
description: Documentación técnica de la arquitectura funcional de Lana Bank, incluyendo arquitectura de aplicación, integraciones, seguridad e infraestructura.
---

# Arquitectura Funcional de Lana Bank

## Tabla de Contenidos

1. [Descripción General](#1-descripción-general)
2. [Arquitectura de la Aplicación](#2-arquitectura-de-la-aplicación)
3. [Flujos de Comunicación](#3-flujos-de-comunicación)
4. [Integraciones con Sistemas Externos](#4-integraciones-con-sistemas-externos)
5. [Flujos de Autenticación y Seguridad](#5-flujos-de-autenticación-y-seguridad)
6. [Segmentación de Red por Ambiente](#6-segmentación-de-red-por-ambiente)
7. [Zonas de Seguridad](#7-zonas-de-seguridad)
8. [Auditoría](#8-auditoría)
9. [Flujo de Préstamo Respaldado por Bitcoin](#9-flujo-de-préstamo-respaldado-por-bitcoin)
10. [Portabilidad y Dependencia de Proveedores](#10-portabilidad-y-dependencia-de-proveedores)
11. [Servidores / Instancias](#11-servidores--instancias)
12. [Sistemas Operativos](#12-sistemas-operativos)
13. [Bases de Datos](#13-bases-de-datos)
14. [Middleware / Integración](#14-middleware--integración)
15. [Servicios Externos](#15-servicios-externos)

---

## 1. Descripción General

Este documento describe la arquitectura lógica de Lana Bank, incluyendo la arquitectura interna de la aplicación, integraciones con sistemas externos, flujos de autenticación y seguridad, segmentación de red por ambiente y zonas de seguridad.

### 1.1 Visión General

Lana Bank es una aplicación de core bancario especializada en **préstamos respaldados por Bitcoin**. La arquitectura sigue los principios de **Domain-Driven Design (DDD)** y **Arquitectura Hexagonal**, separando claramente las capas de dominio, aplicación e infraestructura.

El backend está desarrollado en Rust, usando PostgreSQL como base de datos principal y **Cala Ledger** como motor de contabilidad de partida doble con fuertes garantías de consistencia. Los frontends web están construidos con Next.js y TypeScript, consumiendo APIs GraphQL expuestas por el backend. Para reportes y analítica, existe un pipeline de datos basado en Meltano que extrae información a BigQuery, donde los datos se transforman con dbt.

---

## 2. Arquitectura de la Aplicación

### 2.1 Módulos del Core Bancario

Los módulos del core implementan la lógica de negocio del banco, siguiendo principios de Event Sourcing donde cada entidad mantiene su estado como una secuencia de eventos inmutables.

#### 2.1.1 Crédito

El módulo de crédito es el corazón del sistema, gestionando el ciclo de vida completo de préstamos respaldados por Bitcoin. Una facilidad de crédito pasa por un ciclo de vida bien definido que comienza cuando un operador crea una **CreditFacilityProposal** para un cliente. Esta propuesta entra automáticamente en un proceso de aprobación gestionado por el módulo de gobernanza; los miembros del comité asignado deben votar para aprobarla.

Una vez aprobada, la propuesta se transforma en una **PendingCreditFacility**. En esta etapa, el cliente debe depositar el colateral Bitcoin requerido. Si la facilidad tiene un custodio asignado, los webhooks del custodio mantienen automáticamente el balance del colateral sincronizado. Si no hay custodio (modo manual), un operador puede actualizar el colateral directamente. El sistema monitorea continuamente la relación de colateralización (CVL - Collateral Value to Loan) comparándola con el precio actual de Bitcoin.

La facilidad se activa automáticamente cuando el CVL alcanza el umbral inicial configurado en los términos. **TermValues** definen todos los parámetros del préstamo: la tasa de interés anual, duración (clasificada como corto o largo plazo dependiendo de si excede 12 meses), intervalos de acumulación de interés (diario o mensual), la comisión inicial (cargo único), y tres umbrales críticos de CVL que deben mantener una jerarquía estricta: el CVL inicial debe ser mayor que el CVL de margin call, que a su vez debe ser mayor que el CVL de liquidación. También se configura la política de desembolso, que puede ser única o múltiple.

Con la **CreditFacility** activa, el cliente puede solicitar **Disbursals**. Cada desembolso pasa por su propio proceso de aprobación. Cuando se ejecuta, los fondos se acreditan a la cuenta de depósito del cliente y se crea una **Obligation** representando la deuda. Las obligaciones tienen un ciclo de estados: comienzan como "no vencidas", pasan a "vencidas" en la fecha de vencimiento, pueden convertirse en "morosas" si no se pagan a tiempo, entran en "liquidación" si la morosidad persiste, y finalmente ser marcadas como "incumplidas".

El sistema ejecuta jobs periódicos para la acumulación de intereses. Los **InterestAccrualCycles** calculan intereses según los intervalos configurados y generan nuevas obligaciones por intereses acumulados. Cuando el cliente realiza un **Payment**, el sistema automáticamente asigna fondos a las obligaciones pendientes en orden de prioridad a través de **PaymentAllocation**, típicamente priorizando las obligaciones más antiguas e intereses sobre el principal.

Si el CVL cae por debajo del umbral de margin call, la facilidad entra en estado de alerta. Si cae por debajo del umbral de liquidación, se inicia un **LiquidationProcess** donde el banco puede ejecutar el colateral para recuperar la deuda. El sistema implementa un buffer de histéresis para evitar oscilaciones frecuentes entre estados cuando el CVL está cerca de los umbrales.

#### 2.1.2 Depósito

El módulo de depósitos gestiona las cuentas donde los clientes mantienen sus fondos en USD. Cuando se crea una **DepositAccount** para un cliente, el sistema genera automáticamente las cuentas contables correspondientes en el libro mayor. La categorización contable depende del tipo de cliente: las cuentas para individuos, entidades gubernamentales, empresas privadas, bancos, instituciones financieras y empresas no domiciliadas se agrupan bajo diferentes nodos del plan de cuentas.

Los **Deposits** representan entradas de fondos a la cuenta y se registran inmediatamente. Los **Withdrawals** siguen un flujo más controlado: cuando se inician, los fondos se reservan en contabilidad y se crea un proceso de aprobación. El comité asignado debe aprobar el retiro antes de que se ejecute. Si se aprueba, los fondos salen de la cuenta; si se rechaza o cancela, la reserva se revierte. También existe la posibilidad de revertir depósitos ya registrados cuando sea necesario.

Las cuentas pueden estar en diferentes estados que afectan las operaciones permitidas. Una cuenta **activa** permite todas las operaciones normales. Una cuenta **congelada** previene nuevas operaciones pero mantiene el balance visible; esto es útil para situaciones de cumplimiento donde los fondos necesitan ser bloqueados temporalmente. Una cuenta **cerrada** es permanente y solo se permite si el balance es cero. El módulo también soporta la actualización masiva del estado de todas las cuentas de un cliente, por ejemplo cuando cambia su verificación KYC.

El historial de la cuenta puede consultarse a través del libro mayor, mostrando todas las transacciones que han afectado el balance. El módulo calcula el balance disponible considerando los retiros pendientes de aprobación.

#### 2.1.3 Cliente

Este módulo gestiona la información de los clientes del banco y es fundamental para el cumplimiento regulatorio. Cada cliente se crea con un tipo específico que determina su tratamiento contable y regulatorio: **Individual** para personas naturales, **GovernmentEntity** para entidades gubernamentales, **PrivateCompany** para empresas privadas, **Bank** para bancos, **FinancialInstitution** para otras instituciones financieras, **ForeignAgencyOrSubsidiary** para agencias extranjeras, y **NonDomiciledCompany** para empresas no domiciliadas.

El proceso de verificación KYC se integra con SumSub. Un cliente comienza en estado **PendingVerification**. Cuando SumSub notifica vía webhook que la verificación fue exitosa, el cliente pasa a **Verified** con un nivel KYC (Básico o Avanzado). Si la verificación falla, permanece en estado **Rejected**. El sistema puede configurarse para requerir verificación antes de permitir la creación de cuentas de depósito o facilidades de crédito.

El módulo gestiona documentos asociados al cliente, almacenándolos en la nube y permitiendo la generación de enlaces de descarga temporales. Los documentos pueden archivarse o eliminarse según sea necesario.

Para el cumplimiento de regulaciones de cuentas inactivas, el sistema rastrea la última actividad de cada cliente. Un job periódico clasifica automáticamente a los clientes según su actividad: **Active** si han tenido actividad reciente (menos de un año), **Inactive** si han estado entre uno y diez años sin actividad, y **Suspended** si exceden diez años. Esta clasificación puede afectar el estado de sus cuentas de depósito.

#### 2.1.4 Custodia

El módulo de custodia proporciona una abstracción sobre múltiples proveedores de custodia de Bitcoin, permitiendo al banco trabajar con diferentes custodios según sus necesidades operativas y regulatorias. El sistema está diseñado con un patrón de plugins donde cada **Custodian** implementa una interfaz común. Actualmente están implementados **BitGo** y **Komainu**, pero la arquitectura permite agregar nuevos custodios sin modificar el resto del sistema.

En cada despliegue, múltiples custodios pueden configurarse y activarse simultáneamente. Cuando se crea una facilidad de crédito, se puede especificar qué custodio gestionará el colateral de esa facilidad particular. Esto permite, por ejemplo, usar diferentes custodios para diferentes segmentos de clientes o jurisdicciones.

Cada custodio gestiona **Wallets** que se asignan a facilidades de crédito para recibir colateral Bitcoin. Los custodios notifican al sistema sobre cambios en los balances de las billeteras a través de webhooks. Cuando llega una notificación, el sistema actualiza el **Collateral** asociado a la facilidad correspondiente y recalcula el CVL. Esta sincronización automática es crítica para mantener una visión precisa del riesgo en tiempo real.

Los webhooks de custodios se reciben en endpoints específicos por proveedor y se validan criptográficamente antes de procesarlos. La configuración de cada custodio incluye las credenciales API necesarias y claves para verificar la autenticidad del webhook. Las claves sensibles se almacenan cifradas.

#### 2.1.5 Contabilidad

El módulo de contabilidad implementa un sistema completo de contabilidad de partida doble, fundamental para cualquier institución financiera regulada. Utiliza **Cala Ledger** como motor subyacente, un crate de Rust especializado que proporciona plantillas de transacciones predefinidas y garantías de consistencia ACID para todas las operaciones contables.

El **ChartOfAccounts** define la estructura jerárquica de cuentas del banco. Puede importarse desde archivos CSV y soporta una estructura de árbol con múltiples niveles. Cada nodo del árbol puede ser una cuenta individual o un grupo que agrega las cuentas de sus hijos. El plan de cuentas se integra con otros módulos: las cuentas de depósito de clientes, facilidades de crédito y cuentas de colateral se crean automáticamente como hijos de nodos apropiados según el tipo de cliente y producto.

Cada **LedgerAccount** tiene un tipo de balance normal (débito o crédito) y puede mantener balances en múltiples monedas (USD y BTC). Las **LedgerTransactions** representan movimientos contables que siempre mantienen balance: el total de débitos es igual al total de créditos. El sistema registra automáticamente transacciones para cada operación de negocio: depósitos, retiros, desembolsos, pagos de préstamos, acumulación de intereses y actualizaciones de colateral.

Para reportes financieros, el módulo genera el **TrialBalance** que lista todas las cuentas con sus balances de débito y crédito, útil para verificar que los libros cuadran. El **BalanceSheet** presenta la posición financiera del banco organizando activos, pasivos y patrimonio. El **ProfitAndLoss** muestra ingresos (principalmente intereses de préstamos) menos gastos para calcular el resultado del período.

El sistema soporta múltiples **FiscalYears** y permite consultar balances y reportes para rangos de fechas específicos. También permite **ManualTransactions** para ajustes contables que no se originan de operaciones automatizadas del sistema.

#### 2.1.6 Gobernanza

El sistema de gobernanza proporciona un framework flexible para implementar flujos de aprobación multifirma en operaciones sensibles. Está diseñado para adaptarse a diferentes estructuras organizacionales y requisitos regulatorios.

Los **Committees** representan grupos de personas autorizadas para tomar decisiones sobre ciertos tipos de operaciones. Un comité puede tener cualquier número de miembros, típicamente usuarios del sistema con roles específicos. El mismo usuario puede pertenecer a múltiples comités.

Las **Policies** definen las reglas de aprobación para cada tipo de proceso. Una política especifica qué comité es responsable de aprobar ese tipo de operación y cuál es el umbral requerido: el número mínimo de votos afirmativos necesarios para aprobar. Por ejemplo, una política para aprobación de desembolsos podría requerir 2 de 3 miembros del comité de crédito.

Cuando se inicia una operación que requiere aprobación, el sistema crea automáticamente un **ApprovalProcess** vinculado a la política correspondiente. El proceso comienza en estado pendiente y registra los votos de los miembros del comité. Un miembro puede votar para aprobar o para denegar (con una razón obligatoria). Cuando se alcanza el umbral de aprobación, el proceso se marca como aprobado y se emite un evento **ApprovalProcessConcluded**. Si algún miembro deniega, el proceso termina inmediatamente como rechazado.

Los eventos de conclusión del proceso de aprobación son consumidos por jobs que ejecutan la operación aprobada o manejan el rechazo. Este diseño desacopla el flujo de aprobación de la ejecución, permitiendo que las aprobaciones se procesen de forma asíncrona.

#### 2.1.7 Acceso

El módulo de acceso implementa control de acceso basado en roles (RBAC) para todos los operadores del sistema. Los **Users** representan a las personas que operan el banco a través del Panel de Administración. Cada usuario tiene un identificador único que se vincula con el sistema de autenticación externo.

Los **Roles** agrupan conjuntos de permisos y se asignan a usuarios. Un usuario puede tener múltiples roles, y sus permisos efectivos son la unión de permisos de todos sus roles. Los **PermissionSets** son colecciones nombradas de permisos específicos que facilitan la configuración de roles comunes.

El sistema de permisos es granular: cada operación en cada módulo tiene un permiso asociado. Por ejemplo, hay permisos separados para leer clientes, crear clientes, aprobar KYC, ver facilidades de crédito, iniciar desembolsos, etc. Antes de ejecutar cualquier operación, el sistema verifica que el usuario tenga el permiso correspondiente y registra la acción en el log de auditoría.

El sistema de autorización utiliza **Casbin**, un motor de control de acceso flexible, con políticas almacenadas en PostgreSQL para persistencia y sincronización entre instancias. El modelo RBAC sigue una estructura de tres niveles: Usuario → Rol → PermissionSet → Permisos (Objeto + Acción).

Cada módulo define sus propios conjuntos de permisos que agrupan acciones relacionadas. Los conjuntos de permisos típicos siguen un patrón viewer/writer. El sistema incluye roles predefinidos como Admin (acceso completo), Bank Manager (similar a admin pero sin acceso a gestión de acceso o custodia), y Accountant (enfocado en funciones de contabilidad y visualización).

Los permisos se gestionan dinámicamente a través de la API y los cambios persisten inmediatamente en la base de datos, recargándose en cada verificación de permisos, asegurando que las actualizaciones sean efectivas sin reiniciar el sistema.

#### 2.1.8 Precio

Este módulo obtiene y gestiona precios de Bitcoin, una función crítica para un banco que ofrece préstamos colateralizados con BTC. El sistema se integra con Bitfinex para obtener precios en tiempo real a través de su API.

Cuando se obtiene un nuevo precio, el módulo publica un **CorePriceEvent** que otros módulos consumen. El módulo de crédito es el principal consumidor: usa el precio para calcular el CVL de todas las facilidades activas y determinar si alguna ha caído por debajo de los umbrales de margin call o liquidación. Los cambios de precio pueden disparar actualizaciones de estado en facilidades y potencialmente iniciar procesos de liquidación.

#### 2.1.9 Reportes

El módulo de reportes coordina la generación de reportes regulatorios y operativos. Define tipos de **Report** que especifican qué datos incluir y en qué formato. Cada ejecución de reporte se registra como un **ReportRun** con su estado (pendiente, ejecutando, completado, fallido) y archivos generados.

La generación de reportes se integra con el pipeline de datos: los datos transformados en BigQuery alimentan los reportes finales. El sistema puede integrarse con sistemas de reportes externos según las necesidades regulatorias de cada jurisdicción donde opera el banco.

#### 2.1.10 Módulos de Soporte

Además de los módulos principales, hay módulos de soporte: **document-storage** para almacenamiento de documentos en la nube, **public-id** para generar identificadores públicos legibles para entidades, y **core-money** que define primitivas monetarias (UsdCents, Satoshis) usadas en todo el sistema.

### 2.2 Capa de Aplicación

El directorio `lana/` contiene la capa de aplicación que orquesta los módulos del core y expone funcionalidad externamente.

#### 2.2.1 Servidores GraphQL

El sistema expone dos servidores GraphQL independientes. El **admin-server** sirve al panel de administración usado por operadores del banco, mientras que el **customer-server** sirve al portal de clientes. Ambos servidores incluyen playground integrado para desarrollo y reciben webhooks de servicios externos.

#### 2.2.2 Servicios de Aplicación

El servicio principal **lana-app** orquesta la inicialización de todos los módulos y proporciona el punto de entrada unificado. **lana-cli** ofrece una interfaz de línea de comandos para operaciones administrativas.

Existen servicios especializados para diferentes funciones: **notification** maneja el envío de emails, **pdf-generation** genera contratos PDF, **customer-sync** y **deposit-sync** sincronizan datos con sistemas externos, **user-onboarding** gestiona el registro de operadores, y **dashboard** calcula métricas agregadas. Para desarrollo y pruebas, **sim-bootstrap** permite inicializar datos de simulación.

#### 2.2.3 Sistema de Eventos

El módulo **lana-events** define el enum unificado **LanaEvent** que agrupa todos los eventos de dominio del sistema, permitiendo que el sistema de outbox y los jobs procesen eventos de cualquier módulo de forma uniforme.

### 2.3 Frontends Web

#### 2.3.1 Panel de Administración

El Panel de Administración es la interfaz principal para operadores y personal del banco. Permite gestionar clientes y sus procesos KYC, administrar facilidades de crédito en todas sus etapas, aprobar desembolsos y retiros, y gestionar cuentas de depósito. También proporciona acceso a visualización contable completa (balance, estado de resultados, balance de comprobación), configuración de comités y políticas de aprobación, gestión de usuarios y roles, y generación de reportes regulatorios.

#### 2.3.2 Portal de Clientes

El Portal de Clientes está orientado a los clientes del banco. Actualmente ofrece funcionalidad de solo lectura, permitiendo visualización de facilidades de crédito, estado de desembolsos e historial de transacciones. La arquitectura permite extenderlo en el futuro para soportar operaciones del lado del cliente.

#### 2.3.3 Shared Web

El módulo **shared-web** contiene componentes de UI compartidos entre ambos portales, asegurando consistencia visual y reduciendo duplicación de código.

---

## 3. Flujos de Comunicación

### 3.1 Event Sourcing y Eventos de Dominio

El sistema usa **Event Sourcing** como patrón arquitectónico central. Cada entidad recibe comandos que generan eventos, estos eventos se persisten en la base de datos como la única fuente de verdad, y el estado actual de la entidad se reconstruye aplicando la secuencia de eventos.

Este diseño proporciona auditabilidad completa (cada cambio se registra), la capacidad de reconstruir el estado en cualquier punto en el tiempo, y la posibilidad de agregar nuevas proyecciones sobre datos históricos.

La comunicación entre módulos ocurre a través de eventos públicos. Cada módulo define sus propios eventos en un enum específico (por ejemplo, **CoreCreditEvent** para el módulo de crédito). Un **Publisher** asociado a cada módulo transforma eventos internos de entidad en eventos públicos que otros módulos pueden consumir.

Los eventos públicos típicos incluyen: creación y aprobación de propuestas de crédito, activación y finalización de facilidades, cambios de colateralización, desembolsos liquidados, acumulación de intereses, creación y transición de obligaciones entre estados (vencida, morosa, incumplida), pagos registrados, y procesos de liquidación. Cada evento incluye timestamps de cuándo se registró y cuándo fue efectivo, permitiendo reconstrucciones precisas del estado en cualquier momento.

### 3.2 Patrón Outbox

Para integraciones con sistemas externos que requieren garantías de entrega, el sistema implementa el **Patrón Outbox**. Cuando un módulo necesita publicar un evento, lo persiste en una tabla outbox dentro de la misma transacción de base de datos que la operación de negocio. Esto garantiza atomicidad: o ambos (la operación y el evento) persisten, o ninguno.

PostgreSQL NOTIFY informa inmediatamente a los listeners cuando hay nuevos eventos, evitando la necesidad de polling.

El sistema soporta dos tipos de eventos en el outbox. Los **eventos persistentes** tienen un identificador único, un número de secuencia global monotónicamente creciente, el payload serializado como JSON, contexto de tracing para correlación distribuida, y timestamp de cuándo se registró. Los **eventos efímeros** no tienen secuencia y se usan para notificaciones en tiempo real que no necesitan durabilidad.

Este diseño garantiza **entrega al menos una vez**: un sistema externo puede consumir eventos con certeza de que no perderá ninguno, aunque podría recibir duplicados que debe manejar de forma idempotente.

### 3.3 Sistema de Jobs Asíncronos

Las operaciones que no deben bloquear el flujo principal se ejecutan a través de un sistema de jobs asíncronos. Los workers corren como procesos separados del servidor principal, permitiendo escalar el procesamiento independientemente de los servidores API.

Los jobs pueden programarse de varias formas: ejecutar inmediatamente, programar para una fecha/hora futura específica, o reprogramar al completar para ejecutar de nuevo. Esta flexibilidad es esencial para los flujos temporales del sistema bancario. Por ejemplo, cuando se crea una obligación, se programa un job para la fecha de vencimiento. Cuando ese job se ejecuta, si la obligación no está pagada, la marca como "vencida" y programa el siguiente job para la fecha de morosidad. La cadena continúa: vencida → morosa → liquidación → incumplida, cada transición programada precisamente según los términos de la facilidad.

Para la acumulación de intereses, un job procesa cada acumulación diaria y automáticamente se reprograma para el día siguiente. Cuando termina un período de acumulación (típicamente a fin de mes), programa un job de ciclo de acumulación que consolida intereses y crea la obligación correspondiente.

Otros jobs procesan streams de eventos del outbox continuamente, manteniendo su estado de ejecución (el último evento procesado) y reprogramándose inmediatamente cuando no hay nuevos eventos para continuar escuchando.

### 3.4 Webhooks Entrantes

Los servicios externos notifican al sistema a través de webhooks. **SumSub** envía notificaciones sobre el ciclo de vida de verificación KYC a `/webhook/sumsub`. Cuando un cliente completa su verificación, SumSub notifica el resultado (aprobado o rechazado). El sistema procesa esta notificación y actualiza el estado KYC del cliente, lo que puede desbloquear la creación de cuentas de depósito o facilidades de crédito según la configuración.

Los **custodios de Bitcoin** (BitGo, Komainu) notifican eventos de billetera a `/webhook/custodian/[provider]`. Cada proveedor tiene su propio formato de webhook que el sistema normaliza. Los eventos típicos incluyen depósitos de Bitcoin a billeteras de colateral. Cuando llega una notificación, el sistema verifica su autenticidad (típicamente vía HMAC), identifica la billetera afectada, actualiza el balance de colateral correspondiente, y recalcula el CVL de la facilidad de crédito asociada. Si el nuevo CVL cruza algún umbral configurado, se actualiza el estado de colateralización y se publican los eventos correspondientes.

Este flujo de webhooks es crítico para la gestión de riesgo en tiempo real. Sin él, el sistema dependería de polling periódico y podría tener visibilidad retrasada de cambios en el colateral, aumentando el riesgo durante caídas del precio de Bitcoin.

### 3.5 Flujo de API GraphQL

Las solicitudes de clientes web siguen este flujo: el cliente envía una solicitud GraphQL con un token JWT. El middleware extrae el subject del token y lo inyecta en el contexto. El resolver invoca el caso de uso correspondiente en lana-app, que primero verifica permisos RBAC y luego ejecuta la operación en el módulo core apropiado. Los eventos generados se publican, y la respuesta retorna al cliente.

---

## 4. Integraciones con Sistemas Externos

La aplicación está diseñada para integrarse con varios servicios externos que proporcionan funcionalidades especializadas. Estos servicios no son parte de la infraestructura desplegada pero son componentes críticos del ecosistema operativo.

**Es importante enfatizar que estos servicios deben configurarse externamente** por el cliente o equipo de operaciones. La aplicación simplemente espera recibir las credenciales, tokens, endpoints y otra información de configuración necesaria para integrarse con estos servicios. La aplicación no gestiona la creación, configuración o administración de cuentas en estos servicios externos; solo consume sus APIs y servicios una vez que están configurados y disponibles.

### 4.1 KYC/KYB y AML (Conozca a su Cliente / Conozca su Negocio / Anti-Lavado de Dinero)

#### 4.1.1 Sumsub

Sumsub se usa para gestionar procesos y datos de KYC (Know Your Customer) y KYB (Know Your Business). Este servicio externo maneja la verificación de identidad de clientes y empresas, incluyendo:

- Validación de documentos de identidad
- Verificación biométrica
- Verificación de documentos corporativos
- Cumplimiento regulatorio
- Onboarding de clientes y empresas
- Verificación continua

Sumsub también satisface necesidades de AML (Anti-Money Laundering) además de proporcionar capacidades KYC/KYB. Sumsub incluye funcionalidades de detección y prevención de lavado de dinero, tales como:

- Verificación de listas de sanciones (OFAC, UN, etc.)
- Análisis de transacciones sospechosas
- Monitoreo de patrones de comportamiento
- Reportes regulatorios automáticos
- Integración con sistemas de cumplimiento

La aplicación se integra con Sumsub a través de su API REST. Para configurar la integración, es necesario configurar una cuenta en el servicio Sumsub, obtener credenciales API (API key, API secret), configurar los endpoints correspondientes (pueden variar por región), y proporcionar estas credenciales y endpoints como parte de la configuración del ambiente.

El flujo de integración funciona así: la aplicación envía solicitudes de verificación a Sumsub a través de su API. Sumsub procesa las solicitudes y realiza las verificaciones necesarias. Los resultados de los procesos de onboarding y verificación continua se reciben vía webhooks en el endpoint `/webhook/sumsub`. Cuando un cliente completa su verificación, SumSub notifica el resultado (aprobado o rechazado), y el sistema procesa esta notificación actualizando el estado KYC del cliente, lo que puede desbloquear la creación de cuentas de depósito o facilidades de crédito según la configuración.

La arquitectura también está preparada para integrar sistemas AML adicionales si es necesario. Las integraciones AML típicamente incluyen las funcionalidades mencionadas arriba. La aplicación puede integrarse con proveedores de servicios AML a través de APIs REST o a través de integración con sistemas de terceros. La configuración seguiría el mismo patrón que otras integraciones externas: las credenciales y endpoints se proporcionan como parte de la configuración del ambiente.

### 4.2 Pasarelas de Pago

**Nota importante:** Las integraciones con pasarelas de pago no están implementadas en la versión actual de Lana. Sin embargo, debido a que Lana es modular en diseño, la arquitectura anticipa que estos elementos eventualmente se agregarán según las necesidades del negocio.

La aplicación está diseñada para integrarse con pasarelas de pago externas para procesar transacciones financieras. Aunque las pasarelas específicas pueden variar por cliente y región, la arquitectura soporta integración con múltiples proveedores.

La aplicación está diseñada para soportar varios tipos de integración:

- Procesamiento de pagos con tarjeta (débito/crédito)
- Transferencias bancarias (ACH, wire transfers, etc.)
- Procesamiento de pagos móviles
- Integración con sistemas de compensación y liquidación

Las pasarelas de pago se integrarían vía APIs REST o SOAP. Las credenciales API, endpoints y configuraciones específicas se proporcionarían como parte de la configuración del ambiente. La aplicación está diseñada para soportar múltiples pasarelas simultáneamente, permitiendo enrutamiento de transacciones según reglas de negocio.

Todas las comunicaciones con pasarelas de pago usarían TLS/SSL para cifrado en tránsito. Las credenciales sensibles se almacenarían como secrets en Kubernetes e inyectarían en contenedores de aplicación vía variables de ambiente o volúmenes montados.

### 4.3 BCR (Banco Central de Reserva)

**Nota importante:** La integración con el Banco Central de Reserva (BCR) no está implementada en la versión actual de Lana. Sin embargo, debido a que Lana es modular en diseño, la arquitectura anticipa que esta integración eventualmente se agregará según las necesidades del negocio.

La aplicación está diseñada para incluir soporte para operaciones con el Banco Central de Reserva (BCR), que es el banco central de El Salvador. Esta integración sería crítica para operaciones bancarias regulatorias.

El sistema está diseñado para soportar varios tipos de operaciones con el BCR:

- Depósitos en el BCR (moneda local y extranjera)
- Operaciones repo con el BCR
- Operaciones de financiamiento con el BCR
- Reportes regulatorios y cumplimiento
- Operaciones de liquidez

La integración con el BCR se haría a través de sistemas de comunicación bancaria estándar (típicamente SWIFT, sistemas de mensajería financiera, o APIs específicas del BCR). La configuración incluiría credenciales de acceso a sistemas del BCR, endpoints de comunicación, certificados digitales para autenticación, y configuración de formato de mensajes (ISO 20022, formatos propietarios, etc.).

Las operaciones del BCR se procesarían a través de workers dedicados que manejarían comunicación asíncrona y procesamiento de respuestas. Los datos de operaciones se registrarían en la base de datos principal e integrarían con el sistema contable.

### 4.4 Fuentes de Datos Regulatorios

**Nota importante:** Las integraciones con fuentes de datos regulatorios no están implementadas en la versión actual de Lana. Sin embargo, debido a que Lana es modular en diseño, la arquitectura anticipa que estos elementos eventualmente se agregarán según las necesidades del negocio.

La aplicación está diseñada para integrarse con múltiples fuentes de datos regulatorios para cumplimiento y reportes. Estas incluirían:

- Sistemas de reportes del banco central
- Sistemas de información crediticia
- Registros públicos (registro de empresas, registro de propiedad, etc.)
- Sistemas gubernamentales de verificación de identidad
- Sistemas de intercambio de información financiera

Las integraciones con fuentes de datos regulatorios se harían a través de:

- APIs REST o SOAP proporcionadas por organismos regulatorios
- Sistemas de mensajería financiera (SWIFT, sistemas propietarios)
- Archivos batch para intercambio de datos
- Portales web con autenticación y scraping automatizado (cuando sea necesario)

Los workers de la aplicación procesarían integraciones con sistemas regulatorios de forma asíncrona. Los datos recibidos se validarían, transformarían y almacenarían en la base de datos. Los reportes regulatorios se generarían automáticamente según los requisitos y se enviarían a través de los canales apropiados.

### 4.5 Observabilidad

#### 4.5.1 Honeycomb

Honeycomb se usa para agregación y explotación de datos OpenTelemetry, así como para generar alertas que se integran con software de gestión de pager/on-call. El sistema usa el protocolo OpenTelemetry (OTEL) para enviar métricas, logs y trazas desde el OpenTelemetry Collector a Honeycomb.

El OpenTelemetry Collector se configura con la API key y dataset de Honeycomb. Los datos se envían automáticamente vía el protocolo OTEL. Aunque actualmente se usa Honeycomb, la aplicación usa el protocolo OTEL estándar, lo que permite migrar a otros proveedores compatibles (Datadog, New Relic, Grafana Cloud, etc.) sin modificaciones significativas.

El sistema está instrumentado para proporcionar visibilidad completa de su comportamiento en producción. OpenTelemetry captura trazas de todas las operaciones, desde recibir una solicitud HTTP hasta la respuesta final. Cada operación significativa crea un span con atributos relevantes. Los spans se propagan a través de llamadas asíncronas y entre servicios, permitiendo reconstruir el flujo completo de una operación.

Las trazas se exportan a Honeycomb, donde pueden analizarse para identificar cuellos de botella, errores y patrones de uso. La propagación del contexto de tracing a través del outbox permite correlacionar la operación original con su procesamiento asíncrono posterior.

El logging usa el crate **tracing** de Rust, que proporciona logs estructurados con niveles (error, warn, info, debug, trace) y campos tipados. Los logs se emiten en formato JSON en producción, facilitando su indexación y búsqueda. Cada entrada de log incluye automáticamente el contexto del span actual, conectándola con la traza distribuida.

### 4.6 Almacenamiento de Datos para Reportes

#### 4.6.1 BigQuery

BigQuery se usa como almacenamiento de datos analíticos y de reportes. El sistema usa BigQuery para almacenar datos transformados de las bases de datos operativas PostgreSQL, permitiendo análisis y reportes sin impactar el rendimiento de la base de datos transaccional.

La aplicación usa BigQuery en conjunto con herramientas ETL (Meltano) y transformación de datos (dbt) para cargar y transformar datos desde PostgreSQL a BigQuery. Meltano extrae datos de múltiples fuentes: el extractor principal **tap-postgres** obtiene eventos y entidades del core bancario, y extractores adicionales obtienen precios históricos de Bitfinex y datos de verificación KYC de SumSub.

Los datos se cargan en BigQuery, donde dbt los transforma a través de capas: staging (limpieza de datos crudos), intermediate (lógica de negocio), y outputs (reportes finales). El sistema genera reportes regulatorios que pueden integrarse con sistemas externos según las necesidades de cada jurisdicción.

La configuración incluye JSON de service account, project ID y nombres de datasets. **Es importante notar que, aunque actualmente se usa BigQuery, la aplicación puede refactorizarse para realizar el mismo trabajo en otras bases de datos analíticas.** El código ETL y de transformación puede adaptarse para trabajar con alternativas como Amazon Redshift, Snowflake, Azure Synapse Analytics, o incluso bases de datos analíticas on-premise.

---

## 5. Flujos de Autenticación y Seguridad

### 5.1 IAM (Identity and Access Management)

#### 5.1.1 Keycloak

Keycloak actúa como el servidor central de identidad y acceso (IAM) integrado con la aplicación. Proporciona:

- Gestión de usuarios y roles
- Autenticación a través de múltiples métodos (usuario/contraseña, OAuth2, OIDC)
- Autorización basada en roles (RBAC)
- Single Sign-On (SSO)
- Gestión de sesiones
- Integración con proveedores de identidad externos (Google, etc.)

**Naturaleza Federada y Autenticación Externa de Empleados:**

Debido a su naturaleza federada, Keycloak está diseñado para delegar la autenticación de usuarios internos (empleados) a sistemas de identidad externos. **Se espera que el backend de autenticación de empleados venga externamente.** Por ejemplo, si la institución usa Azure Active Directory (Azure AD), Keycloak debería integrarse con Azure AD para que Keycloak delegue la autenticación a Azure AD. Este es un detalle de despliegue que debe abordarse en cada caso según las necesidades de la institución y los sistemas de identidad existentes.

**Configurabilidad:**

Keycloak es altamente configurable y la configuración descrita a continuación es una sugerencia que puede adaptarse a las necesidades de cada despliegue. Realms, clientes, flujos de autenticación y proveedores de identidad pueden configurarse según los requisitos específicos de cada cliente.

Como sugerencia, se configuran tres realms:

- **Internal Realm:** Para usuarios internos y servicios de aplicación
- **Customer Realm:** Para clientes de la aplicación
- **Data-Dagster Realm:** Para acceso a herramientas de datos (Dagster)

Similarmente, se sugieren tres clientes de aplicación:

- **internal-service-account:** Para servicios internos de la aplicación
- **customer-service-account:** Para el portal de clientes
- **oauth2-proxy:** Para autenticación OAuth2 Proxy

El flujo de autenticación para usuarios internos funciona así: cuando un usuario accede al Panel de Administración (`admin.{domain}`), la aplicación redirige a Keycloak para autenticación. Keycloak puede delegar la autenticación a un proveedor de identidad externo (ej. Azure AD, LDAP, etc.) o validar credenciales directamente. Después de autenticación exitosa, Keycloak genera tokens JWT usados para autenticar solicitudes a la API GraphQL. Finalmente, Oathkeeper valida los tokens JWT antes de permitir acceso a recursos.

Para clientes, el flujo de autenticación es similar: cuando un cliente accede al Portal de Clientes (`app.{domain}`), la aplicación redirige a Keycloak (Customer Realm) para autenticación. Keycloak valida credenciales y genera tokens JWT, que se usan para autenticar solicitudes a la API pública. Oathkeeper valida los tokens JWT antes de permitir acceso a recursos.

Los flujos de autenticación descritos son ejemplos y pueden variar según la configuración específica de cada despliegue, especialmente respecto a la integración con proveedores de identidad externos para usuarios internos.

#### 5.1.2 Oathkeeper

Oathkeeper actúa como proxy de autenticación y autorización, proporcionando:

- Validación de tokens JWT
- Enrutamiento de solicitudes autenticadas
- Mutación de tokens (transformación de claims)
- Reglas de acceso basadas en URL y método HTTP
- Alta disponibilidad (2 réplicas por defecto)

Se configuran varias reglas de acceso:

- **admin-api:** Protege el endpoint GraphQL del Panel de Administración, requiere autenticación JWT
- **admin-ui:** Protege la interfaz del Panel de Administración, permite acceso sin autenticación (autenticación manejada por la aplicación)
- **customer-ui:** Protege el Portal de Clientes, permite acceso sin autenticación (autenticación manejada por la aplicación)
- **customer-api:** Protege la API pública del Portal de Clientes, requiere autenticación JWT

El flujo de validación funciona así: cuando un cliente envía una solicitud con un token JWT en el header Authorization, Oathkeeper extrae y valida el token JWT contra el JWKS de Keycloak. Oathkeeper verifica que el token no haya expirado y que el issuer sea válido, luego aplica reglas de autorización según URL y método. Si la autorización es exitosa, Oathkeeper muta el token (opcional) y reenvía la solicitud al servicio upstream.

#### 5.1.3 OAuth2 Proxy

OAuth2 Proxy proporciona autenticación OAuth2/OIDC para aplicaciones que no soportan autenticación nativa. Se usa principalmente para proteger acceso a Dagster.

El flujo de autenticación con OAuth2 Proxy funciona así: cuando un usuario accede a Dagster (`dagster.{domain}`), OAuth2 Proxy intercepta la solicitud y verifica si hay una sesión válida. Si no hay sesión, OAuth2 Proxy redirige a Keycloak para autenticación. El usuario se autentica en Keycloak (puede usar Google como proveedor de identidad), y Keycloak redirige de vuelta a OAuth2 Proxy con un código de autorización. OAuth2 Proxy intercambia el código por tokens y crea una sesión, finalmente permitiendo acceso a Dagster con headers de autenticación.

### 5.2 WAF (Web Application Firewall)

El sistema usa NGINX Ingress Controller como punto de entrada, que proporciona capacidades WAF a través de varias funcionalidades.

**Geo-blocking:** Permite bloquear países no soportados configurados vía GeoIP2. La base de datos GeoIP2 se actualiza automáticamente desde un bucket GCS, y las reglas de bloqueo se configuran vía mapas NGINX.

**Rate Limiting:** Incluye limitación de solicitudes por minuto por host, limitación de conexiones simultáneas, y configuración por host (portal de clientes, panel de administración, dagster).

**Protección Adicional:** Existe la posibilidad de configurar WAF vía anotaciones NGINX, integración con servicios WAF externos (Cloudflare, AWS WAF, Azure WAF, etc.), y protección contra ataques comunes (DDoS, SQL injection, XSS, etc.).

### 5.3 Firewalls

#### 5.3.1 Reglas de Firewall en GCP

Las reglas de firewall en GCP incluyen:

- **Intra-cluster Egress:** Permite comunicación entre pods y con el master (protocolos TCP, UDP, ICMP, SCTP, ESP, AH) a Master CIDR, subred del Cluster, rango de Pods y rango de Services
- **Webhook Ingress:** Permite al master llamar webhooks en pods (puertos 8443, 443) desde Master CIDR
- **DMZ to Nodes:** Permite acceso desde bastion a nodos del cluster (todos los protocolos) desde subred DMZ

#### 5.3.2 Network Security Groups en Azure

Los Network Security Groups (NSG) en Azure proporcionan reglas de firewall por subred:

- **PostgreSQL NSG:** Permite solo tráfico desde VirtualNetwork al puerto 5432
- **Cluster NSG:** Controla tráfico hacia y desde nodos de Kubernetes
- **DMZ NSG:** Controla acceso a hosts bastion

### 5.4 Cifrado en Tránsito

Para comunicaciones externas, todos los servicios expuestos públicamente usan HTTPS/TLS. Los certificados SSL/TLS se gestionan automáticamente por cert-manager, que puede usar Let's Encrypt (para certificados públicos) o una CA interna (para certificados privados). Los certificados se renuevan automáticamente antes de expirar.

Para comunicaciones internas, las bases de datos PostgreSQL requieren SSL/TLS para todas las conexiones (`sslmode = "require"` en Azure). Las comunicaciones entre servicios dentro del cluster pueden usar mTLS (mutual TLS) vía service mesh (opcional). Las comunicaciones con servicios externos (Sumsub, pasarelas de pago, etc.) usan HTTPS/TLS.

Se usan protocolos y versiones seguros: TLS 1.2 o superior para todas las conexiones, cipher suites seguros configurados en NGINX Ingress, y Perfect Forward Secrecy (PFS) habilitado.

### 5.5 Cifrado en Reposo

Las bases de datos gestionadas (Cloud SQL, Azure PostgreSQL) usan cifrado en reposo proporcionado por el proveedor cloud. En GCP, Cloud SQL usa cifrado automático de datos en reposo. En Azure, Azure PostgreSQL Flexible Server usa cifrado automático con claves gestionadas por Microsoft o claves gestionadas por el cliente (CMK). Los backups también están cifrados.

Los objetos almacenados en buckets GCS (documentos, reportes, etc.) usan cifrado en reposo. El cifrado puede ser gestionado por Google o vía claves gestionadas por el cliente (CMEK).

Los secrets de Kubernetes se almacenan cifrados en etcd. En GCP, etcd está cifrado vía claves gestionadas por Google. En Azure, etcd está cifrado vía claves gestionadas por Microsoft. Los secrets sensibles (contraseñas, API keys, etc.) se almacenan como Kubernetes Secrets y se inyectan en contenedores.

Los volúmenes persistentes usan cifrado proporcionado por el proveedor cloud. En GCP, los Persistent Volumes usan cifrado automático. En Azure, los Managed Disks usan cifrado automático.

### 5.6 VPN (Virtual Private Network)

**Nota importante:** La configuración de VPN es un detalle de despliegue, no parte de la aplicación Lana. Lo presentado a continuación son sugerencias y opciones arquitectónicas que pueden ser útiles para diferentes escenarios. Es responsabilidad del operador tomar las decisiones finales y diseños que se ajusten a las necesidades específicas de su despliegue, incluyendo requisitos de seguridad, cumplimiento y organizacionales.

El sistema puede soportar múltiples opciones de VPN para acceso administrativo y de empleados, dependiendo de la configuración elegida por el operador.

#### 5.6.1 VPN Site-to-Site

Una opción es configurar VPN entre la red de oficina/corporativa y la VPC/VNet vía Cloud VPN o Partner VPN en GCP, o VPN Gateway (Site-to-Site) en Azure. Las ventajas incluyen acceso directo a recursos internos sin exponer servicios a Internet, no se requieren IPs públicas para servicios internos, y control de acceso centralizado. Los empleados conectados a la red corporativa accederían automáticamente.

#### 5.6.2 VPN Client (Point-to-Site)

Otra opción es configuración de VPN cliente para acceso remoto, que presenta diferencias dependiendo del proveedor cloud: Cloud VPN no soporta P2S nativamente en GCP, requiriendo solución de terceros, mientras que en Azure se puede usar VPN Gateway (Point-to-Site) con OpenVPN o IKEv2. Las ventajas incluyen acceso desde cualquier ubicación, autenticación por certificado o usuario/contraseña, y no se requiere red corporativa. Los empleados remotos se conectarían vía cliente VPN.

#### 5.6.3 Bastion Host con VPN

Una alternativa es configuración de VPN al host bastion con port forwarding, que funcionaría así: el empleado se conecta a VPN, VPN termina en el host bastion, y el empleado accede a servicios internos a través del bastion. Las ventajas incluyen control de acceso granular, auditoría centralizada, y no se requieren cambios a la infraestructura principal.

#### 5.6.4 Acceso vía Bastion (SSH Tunneling)

Otra opción es configuración de túnel SSH a través del host bastion para acceso administrativo y debugging. Por ejemplo, tunneling a base de datos PostgreSQL vía `ssh -L localhost:5432:db-internal-ip:5432 bastion-host`. Las ventajas incluyen acceso seguro a recursos internos sin exponerlos a Internet.

El operador debe evaluar estas opciones y seleccionar o diseñar la solución de acceso remoto que mejor se ajuste a sus requisitos específicos de seguridad, cumplimiento y operacionales.

### 5.7 Certificados

cert-manager gestiona automáticamente certificados SSL/TLS. Los certificados se crean como recursos de Kubernetes (Certificates), cert-manager solicita certificados de Let's Encrypt o CA interna según configuración, los certificados se renuevan automáticamente antes de expirar, y se almacenan como Kubernetes Secrets.

Los certificados para comunicación con bases de datos y servicios internos pueden ser gestionados por cert-manager o proporcionados manualmente. Los certificados para autenticación con servicios externos (BCR, sistemas regulatorios) se proporcionan como parte de la configuración del ambiente.

---

## 6. Segmentación de Red por Ambiente

La arquitectura implementa aislamiento completo entre diferentes ambientes (DEV, QA, UAT, PROD). **Los ambientes no comparten ningún recurso de infraestructura.**

Cada ambiente tiene:

- Su propia VPC/VNet completamente aislada
- Su propio cluster de Kubernetes
- Sus propias instancias de base de datos
- Sus propios load balancers e IPs públicas
- Sus propias credenciales y secrets
- Sus propios dominios y certificados SSL/TLS

No hay conectividad de red directa entre ambientes. No hay VPC/VNet peering entre ambientes. No hay rutas de red que permitan comunicación entre ambientes. Cada ambiente es completamente independiente y aislado de los demás.

---

## 7. Zonas de Seguridad

La arquitectura implementa un modelo de zonas de seguridad que segmenta la infraestructura según nivel de exposición y requisitos de seguridad.

### 7.1 Zona Pública

La zona pública contiene servicios que están expuestos a Internet y son públicamente accesibles.

**Componentes:**
- **Load Balancer:** IP pública proporcionada por el proveedor cloud
- **NGINX Ingress Controller:** Punto de entrada para todo el tráfico HTTP/HTTPS
- **Portal de Clientes:** Frontend públicamente accesible (`app.{domain}`)
- **Certificados SSL/TLS:** Gestionados por cert-manager (Let's Encrypt o CA interna)

**Características de Seguridad:**
- TLS/SSL requerido para todas las conexiones (HTTPS)
- Geo-blocking configurado para bloquear países no soportados
- Rate limiting configurado por host
- Capacidades WAF vía NGINX o servicios externos
- Monitoreo y alertas de tráfico anómalo
- Autenticación requerida para acceso a funcionalidades sensibles

El flujo de tráfico sigue esta ruta:
```
Cliente Internet → Load Balancer (IP Pública) → NGINX Ingress Controller → Servicios de Aplicación
```

### 7.2 Zona Privada

La zona privada contiene servicios que no están expuestos a Internet y solo son accesibles desde dentro de la VPC/VNet.

**Componentes:**
- **Cluster de Kubernetes:** Nodos de aplicación y pods
- **Servicios Backend:** APIs internas, workers, servicios de procesamiento
- **Panel de Administración:** Panel administrativo (accesible solo vía VPN o red privada)
- **Bases de Datos PostgreSQL:** Instancias de base de datos con acceso solo privado

**Características de Seguridad:**
- Sin IPs públicas (nodos sin IPs públicas, `enable_private_nodes = true` en GCP)
- Acceso solo desde dentro de la VPC/VNet
- Network Policies habilitadas (Calico en GCP, Azure Network Policy en Azure)
- Reglas de firewall restringiendo comunicación entre componentes
- TLS/SSL para comunicaciones internas
- Autenticación y autorización vía Keycloak y Oathkeeper

El flujo de tráfico sigue esta ruta:
```
Servicios Internos → Network Policies → Servicios de Aplicación → Bases de Datos (PostgreSQL)
```

### 7.3 Zona de Administración

La zona de administración contiene recursos para acceso administrativo y gestión de infraestructura.

**Componentes:**
- **Hosts Bastion:** Hosts en subred DMZ para acceso administrativo
- **API de Kubernetes:** Endpoint privado del cluster (no accesible desde Internet)
- **Herramientas de Gestión:** Helm, kubectl, herramientas CI/CD

**Características de Seguridad:**
- Hosts bastion en subred DMZ aislada
- Acceso a API de Kubernetes restringido a hosts bastion y redes autorizadas
- Autenticación fuerte requerida (claves SSH, certificados)
- Auditoría de acceso administrativo
- Acceso vía VPN o SSH tunneling
- Rotación regular de credenciales y claves

El flujo de acceso sigue esta ruta:
```
Administrador → VPN/SSH → Host Bastion → API de Kubernetes / Servicios Internos
```

### 7.4 Zona de Backups

La zona de backups contiene sistemas y almacenamiento para respaldos de datos.

**Componentes:**
- **Backups de Base de Datos:** Backups automáticos gestionados por el proveedor cloud
- **Almacenamiento de Backups:** Buckets GCS o Azure Blob Storage para backups
- **Point-in-Time Recovery:** Habilitado para bases de datos críticas

**Características de Seguridad:**
- Backups cifrados en reposo
- Backups geo-redundantes (multiregión) para redundancia crítica
- Retención configurable (7-35 días dependiendo del ambiente)
- Acceso restringido a backups (solo servicios autorizados)
- Rotación automática de backups antiguos
- Pruebas periódicas de restauración

En GCP Cloud SQL, los backups automáticos están habilitados, point-in-time recovery está habilitado, y los backups son multiregión. En Azure PostgreSQL, los backups automáticos tienen retención configurable y los backups geo-redundantes son opcionales.

### 7.5 Zona de Monitoreo

La zona de monitoreo contiene sistemas de observabilidad, logging y alertas.

**Componentes:**
- **OpenTelemetry Collector:** Recolecta métricas, logs y trazas
- **Honeycomb:** Agregación y análisis de datos de telemetría
- **Sistemas de Alertas:** Integración con sistemas pager/on-call (Zenduty, PagerDuty, etc.)
- **Logs de Aplicación:** Logs de pods y servicios de Kubernetes

**Características de Seguridad:**
- Comunicación cifrada con servicios externos de monitoreo (TLS/SSL)
- API keys almacenadas como Kubernetes Secrets
- Acceso restringido a dashboards y datos de monitoreo
- Retención de logs configurable
- Anonimización de datos sensibles en logs

El flujo de datos sigue esta ruta:
```
Aplicaciones → OpenTelemetry Collector → Honeycomb → Alertas → Sistemas Pager/OnCall
```

### 7.6 Comunicación Entre Zonas

**Nota importante:** Muchos de los detalles sobre comunicación entre zonas, acceso administrativo, almacenamiento de backups y monitoreo son detalles de despliegue, no parte de la aplicación Lana. Lo presentado a continuación son sugerencias y consideraciones arquitectónicas. El diseño final, implementación y operación de estos aspectos es responsabilidad del operador del despliegue, quien debe adaptarlos a sus requisitos específicos de seguridad, cumplimiento y operacionales.

#### 7.6.1 Definiciones de Zonas

**Zona Pública:** Contiene aplicaciones web e integraciones/APIs accesibles desde WAN (Wide Area Network). El acceso a estas aplicaciones se controla desde INGRESS, que actúa como punto de entrada y aplica reglas de autenticación, autorización y seguridad.

**Zona Privada:** Todos los servicios de aplicación están en la red privada. Estos servicios no están expuestos directamente a Internet y solo son accesibles desde dentro de la VPC/VNet o a través de mecanismos de acceso controlado.

#### 7.6.2 Comunicación de Zona Pública a Zona Privada

La comunicación de la Zona Pública a la Zona Privada se hace vía tráfico HTTP/HTTPS desde Internet pasando por el Load Balancer, luego el NGINX Ingress Controller, y finalmente los Servicios de Aplicación. La autenticación y autorización se realiza vía Oathkeeper antes de acceder a servicios privados.

#### 7.6.3 Acceso a Zona Privada (Detalle de Despliegue)

Cómo organizar el acceso administrativo y de empleados a la Zona Privada (vía hosts Bastion, VPN, SSH tunneling, etc.) es un detalle de despliegue que debe ser diseñado e implementado por el operador según sus necesidades específicas. El operador debe considerar factores como requisitos de seguridad, políticas organizacionales, cumplimiento y preferencias de acceso remoto.

#### 7.6.4 Almacenamiento de Backups (Detalle de Despliegue)

Cómo almacenar backups de forma segura y privada es un detalle de despliegue. El operador debe diseñar e implementar la estrategia de backups que mejor se ajuste a sus requisitos, incluyendo consideraciones sobre cifrado, redundancia geográfica, retención y acceso restringido.

#### 7.6.5 Monitoreo (Detalle de Despliegue)

La configuración y operación de sistemas de monitoreo, observabilidad y alertas es un detalle de despliegue. El operador debe seleccionar e implementar las herramientas y servicios de monitoreo que mejor se ajusten a sus necesidades, incluyendo consideraciones sobre dónde almacenar métricas y logs, cómo configurar alertas y qué nivel de observabilidad se requiere.

#### 7.6.6 Restricciones de Comunicación

Existen restricciones importantes en el diseño de zonas: típicamente no hay comunicación directa de la zona pública a la zona de administración, no hay comunicación directa de la zona pública a la zona de backups, y la comunicación entre zonas está controlada por reglas de firewall y network policies. Sin embargo, el diseño específico de estas restricciones y controles es responsabilidad del operador del despliegue.

---

## 8. Auditoría

El sistema de auditoría es un componente transversal que registra todas las acciones realizadas en el sistema, proporcionando trazabilidad completa para cumplimiento regulatorio e investigación de incidentes.

### 8.1 Estructura de Entrada de Auditoría

Cada entrada de auditoría captura cuatro dimensiones fundamentales:

- **Subject:** Quién realizó la acción. Puede ser un usuario identificado por su UUID (cuando opera a través del Panel de Administración) o el sistema mismo (para operaciones automáticas como jobs de acumulación de intereses o procesamiento de webhooks).

- **Object:** Sobre qué se realizó la acción. Los objetos son tipados y pueden referirse a entidades específicas (un cliente particular, una facilidad de crédito específica) o categorías completas (todos los clientes, todas las facilidades). El formato incluye el tipo de entidad y su identificador, por ejemplo `customer/550e8400-e29b-41d4-a716-446655440000` o `credit-facility/all`.

- **Action:** Qué tipo de operación se intentó. Las acciones se categorizan por módulo y entidad, siguiendo un formato como `customer:read`, `credit-facility:create`, `withdrawal:approve`. Cada módulo define sus propias acciones posibles.

- **Authorized:** Si la operación fue permitida o denegada. El sistema registra incluso intentos de acceso fallidos, lo que permite detectar patrones de intentos no autorizados.

Adicionalmente, cada entrada tiene un timestamp de cuándo se registró y un identificador secuencial único.

### 8.2 Integración con Flujo de Operaciones

El sistema de auditoría está directamente integrado en el flujo de autorización. Cuando un usuario intenta realizar una operación, el sistema de permisos (RBAC) verifica si tiene los permisos necesarios y simultáneamente registra la entrada de auditoría. Esta integración garantiza que ninguna operación, exitosa o fallida, escape del registro.

Para operaciones que ocurren dentro de transacciones de base de datos, el sistema soporta registro de auditoría transaccional: la entrada de auditoría se inserta en la misma transacción que la operación de negocio, garantizando consistencia. Si la transacción falla, la entrada de auditoría también se revierte.

Las operaciones del sistema (no iniciadas por usuarios) se registran con un subject especial "system", permitiendo distinguir entre acciones humanas y automatizadas. Esto es importante para operaciones como transición automática de obligaciones a estado "vencida" o "morosa", sincronización de colateral desde webhooks de custodios, o acumulación de intereses.

### 8.3 Correlación con Tracing

El sistema de auditoría se integra con el contexto de tracing distribuido. Cuando se registra una entrada de auditoría, se asocia con el span actual de OpenTelemetry. Esto permite correlacionar una entrada de auditoría específica con la traza completa de la operación, incluyendo todas las llamadas internas, consultas a base de datos y comunicaciones con servicios externos que ocurrieron como parte de esa operación.

### 8.4 Consulta del Log de Auditoría

El log de auditoría es consultable a través de la API GraphQL del Panel de Administración, permitiendo a operadores autorizados (con el permiso `audit:list`) buscar y filtrar entradas. La paginación es basada en cursor para manejar eficientemente grandes volúmenes de datos. Las entradas se ordenan por ID descendente, mostrando las más recientes primero.

---

## 9. Flujo de Préstamo Respaldado por Bitcoin

Para ilustrar cómo interactúan los módulos, este es el flujo típico de un préstamo:

### 9.1 1. Propuesta y Aprobación

1. **PROPUESTA:** El cliente solicita una propuesta de crédito, que entra en un proceso de aprobación gestionado por el módulo de gobernanza.

2. **APROBACIÓN:** El comité asignado vota para aprobar la propuesta. Cuando se alcanza el umbral de aprobación, se crea una **PendingCreditFacility**.

### 9.2 2. Colateralización y Activación

3. **COLATERALIZACIÓN:** El cliente deposita Bitcoin como colateral a través del custodio configurado. Los webhooks del custodio mantienen automáticamente el balance de colateral sincronizado, y el sistema recalcula el CVL.

4. **ACTIVACIÓN:** Cuando el CVL alcanza el umbral inicial configurado en los términos, la facilidad se activa automáticamente.

### 9.3 3. Desembolsos y Vida del Préstamo

5. **DESEMBOLSO:** El cliente puede solicitar desembolsos, cada uno pasando por su propio proceso de aprobación. Cuando se ejecuta, los fondos se acreditan a la cuenta de depósito del cliente y se crea una **Obligation** representando la deuda.

6. **VIDA DEL PRÉSTAMO:** Jobs periódicos calculan y registran intereses acumulados según los intervalos configurados, generando nuevas obligaciones por intereses acumulados.

7. **PAGOS:** Cuando el cliente realiza un **Payment**, el sistema automáticamente asigna fondos a las obligaciones pendientes en orden de prioridad vía **PaymentAllocation**, típicamente priorizando las obligaciones más antiguas e intereses sobre el principal.

8. **CIERRE:** Cuando todas las obligaciones están liquidadas, la facilidad puede cerrarse y el colateral Bitcoin se libera al cliente.

### 9.4 4. Monitoreo de CVL y Gestión de Riesgo

A lo largo de todo el ciclo de vida, el sistema monitorea continuamente el CVL. Si cae por debajo del umbral de margin call, la facilidad entra en estado de alerta. Si cae por debajo del umbral de liquidación, se inicia un **LiquidationProcess** donde el banco puede ejecutar el colateral para recuperar la deuda.

### 9.5 Notas Adicionales

#### 9.5.1 Configuración del Cliente

Diferentes clientes pueden tener configuraciones específicas de integraciones externas, zonas de seguridad y segmentación de red según sus requisitos regulatorios y de negocio.

#### 9.5.2 Actualizaciones y Mantenimiento

Las actualizaciones de componentes de seguridad (Keycloak, Oathkeeper, cert-manager, etc.) se gestionan vía Helm charts y se aplican de forma controlada en cada ambiente.

#### 9.5.3 Cumplimiento Regulatorio

La arquitectura está diseñada para cumplir con requisitos regulatorios bancarios, incluyendo:

- Aislamiento de datos por ambiente
- Cifrado de datos en tránsito y en reposo
- Auditoría de acceso y logging
- Backups y recuperación ante desastres
- Integración con sistemas regulatorios

---

## 10. Portabilidad y Dependencia de Proveedores

### 10.1 Portabilidad del Cluster de Kubernetes

La aplicación está diseñada para desplegarse en un cluster de Kubernetes **agnóstico a cualquier proveedor cloud**. No hay dependencia de proveedor por usar servicios propietarios de ningún proveedor, ya que la arquitectura usa componentes y servicios estándar de Kubernetes que pueden reemplazarse por alternativas equivalentes. Algunos servicios no gestionados vía Kubernetes, como bases de datos Postgres, pueden desplegarse en hosts genéricos.

La aplicación **puede desplegarse on-premise** sin modificaciones significativas. Sin embargo, el gestor de infraestructura física necesitará abordar aspectos críticos que los proveedores cloud gestionan automáticamente: implementar estrategias de backup equivalentes, asegurar alta disponibilidad a través de redundancia de hardware y componentes, implementar replicación de datos a ubicaciones geográficamente separadas (replicación offsite), y gestionar mantenimiento de hardware, actualizaciones y monitoreo.

Los componentes principales son portables y pueden ejecutarse en cualquier ambiente compatible con Kubernetes:

- Cluster de Kubernetes (cualquier distribución: GKE, AKS, EKS, Rancher, k3s, etc.)
- PostgreSQL (Cloud SQL, Azure PostgreSQL, o instancias gestionadas on-premise)
- Ingress Controller (NGINX Ingress)
- Helm Charts (estándar de Kubernetes)
- Aplicaciones containerizadas (Docker)

Los componentes específicos del proveedor pueden reemplazarse: Load Balancers con soluciones on-premise o alternativas, Persistent Volumes pueden usar cualquier storage class compatible con Kubernetes, y VPC/VNet puede reemplazarse por redes físicas o SDN (Software Defined Networking).

---

## 11. Servidores / Instancias

### 11.1 Tipos de Instancia

**Actualmente**, la infraestructura se despliega en dos proveedores cloud: **Google Cloud Platform (GCP)** y **Microsoft Azure**. Para estos dos proveedores podemos ofrecer consejos muy específicos y configuraciones detalladas, ya que son los ambientes en los que tenemos experiencia operativa directa. Sin embargo, la arquitectura es portable y la aplicación puede ajustarse para otros proveedores (como AWS, Oracle Cloud, etc.) o sistemas on-premise.

#### 11.1.1 Instancias de Aplicación (Nodos de Kubernetes)

En GCP, los nodos de Kubernetes usan el tipo de máquina **n2-standard-4** por defecto, proporcionando 4 vCPUs, 16 GB de RAM, y 100 GB de disco (pd-standard). El cluster está configurado con autoscaling permitiendo entre 1 y 3 nodos (configurable por ambiente). Los nodos usan la imagen COS_CONTAINERD (Container-Optimized OS con containerd) y se distribuyen automáticamente entre múltiples zonas dentro de la región para redundancia.

En Azure, los nodos usan **Standard_DS2_v2** por defecto (2 vCPUs, 7 GB RAM, disco SSD premium) o **Standard_B1s** para ambientes de desarrollo/staging (1 vCPU, 1 GB RAM). El autoscaling también permite entre 1 y 3 nodos según configuración.

#### 11.1.2 Instancias de Base de Datos

Las bases de datos usan una arquitectura de instancia única por base de datos, diseñada para escalado vertical (aumentar CPU, RAM y almacenamiento) en lugar de escalado horizontal. Se recomienda activar opciones de autoscaling del proveedor cloud para expandir almacenamiento proporcionalmente al crecimiento de la base de datos.

En GCP, las instancias usan Cloud SQL para PostgreSQL (Enterprise Edition) con tier por defecto de **db-custom-1-3840** (1 vCPU, 3.75 GB RAM). El almacenamiento comienza en 100 GB y debe expandirse según uso. La alta disponibilidad es configurable vía `highly_available = true/false`, permitiendo modo ZONAL (sin redundancia) o REGIONAL (con redundancia entre zonas).

En Azure, las instancias usan Azure Database for PostgreSQL Flexible Server con SKU por defecto **GP_Standard_D2s_v3** (2 vCPUs, 8 GB RAM). El almacenamiento también comienza en 100 GB y es expandible. La alta disponibilidad se configura vía `geo_redundant_backup_enabled` y las instancias se ubican en Zona 1 por defecto (configurable).

#### 11.1.3 Instancias Bastion

Los hosts bastion proporcionan acceso administrativo seguro a la infraestructura. En GCP usan tipo de máquina **e2-small** (2 vCPUs compartidas, 2 GB RAM) con Ubuntu 22.04 LTS. En Azure usan **Standard_DS1_v2** (1 vCPU, 3.5 GB RAM, 7 GB SSD) también con Ubuntu 22.04 LTS.

#### 11.1.4 Instancias de Aplicación (Stateless)

Los elementos stateless (servidor backend, servidor auth, servidores front de aplicación, workers, etc.) corren como pods en Kubernetes y pueden escalar horizontalmente. Se recomienda comenzar con 1 réplica por servicio y aumentar el número de réplicas según la carga necesaria.

Los workers tienen recursos configurados con requests de 1000m CPU (1 core) y 1000Mi-1500Mi de memoria, con limits de 2000m-3000m CPU y 3000Mi-4000Mi de memoria, dependiendo del ambiente.

### 11.2 Almacenamiento Estimado

Los nodos de Kubernetes usan 100 GB de disco de sistema por nodo (pd-standard en GCP), resultando en almacenamiento total estimado de 100-300 GB dependiendo del número de nodos.

Para bases de datos, se recomienda comenzar con 100 GB por instancia. El almacenamiento es proporcional al crecimiento de la base de datos, y **se recomienda activar opciones de autoscaling del proveedor cloud** para expandir almacenamiento automáticamente con el uso de la aplicación. En GCP Cloud SQL no hay límite específico configurado en código (depende del tier), mientras que en Azure PostgreSQL es configurable comenzando con 100 GB.

Los volúmenes persistentes se crean según sea necesario para aplicaciones específicas (por ejemplo, Meltano). Es importante notar que toda la persistencia se gestiona con PostgreSQL; no se usan otros sistemas de almacenamiento persistente como Redis o MongoDB.

### 11.3 Redundancia

Los clusters de Kubernetes en GCP distribuyen automáticamente nodos entre múltiples zonas dentro de la región. En Azure, los nodos se distribuyen en Availability Sets/Zones según configuración. Ambos proveedores tienen auto-repair habilitado, pero auto-upgrade está deshabilitado para permitir upgrades manuales controlados.

Para bases de datos, **es crítico activar backups redundantes multiregión** para evitar pérdida de datos que podría ser desastrosa para operaciones bancarias. En GCP, esto se logra a través de alta disponibilidad con `availability_type = "REGIONAL"` (cuando está habilitado), point-in-time recovery habilitado, backups automáticos habilitados, y configuración de backup multiregión para redundancia crítica. En Azure, `geo_redundant_backup_enabled = true` debe estar habilitado, con retención de backup configurable entre 7-35 días.

Los servicios de aplicación corren como deployments de Kubernetes con múltiples réplicas cuando es necesario. Por ejemplo, Oathkeeper tiene 2 réplicas por defecto para alta disponibilidad.

---

## 12. Sistemas Operativos

### 12.1 Versiones Compatibles y Certificadas

El sistema está diseñado para ejecutarse en ambientes Linux. Todo se gestiona con imágenes Docker que usan Nix para crear ambientes determinísticos, asegurando reproducibilidad y consistencia entre diferentes ambientes.

Los nodos de Kubernetes en GCP usan Container-Optimized OS (COS) con containerd, con versión específica gestionada por GKE y compatible con Kubernetes 1.32.9-gke.1548000 (versión por defecto). En Azure, los nodos usan Ubuntu (versión gestionada por AKS) compatible con Kubernetes 1.30.9 (versión por defecto).

Los hosts bastion usan **Ubuntu 22.04 LTS** (Jammy Jellyfish), certificado y probado en ambos proveedores. En GCP se usa la imagen `ubuntu-2204-lts` y en Azure `0001-com-ubuntu-server-jammy`.

Los contenedores CI/CD usan Ubuntu como base (sin especificar versión LTS específica en Dockerfile) y también usan Docker con Nix para ambientes determinísticos.

---

## 13. Bases de Datos

### 13.1 Tipo y Versión

El sistema usa exclusivamente **PostgreSQL** como sistema de gestión de base de datos. La versión recomendada varía por proveedor: en GCP se usa **PostgreSQL 17** (POSTGRES_17) como versión por defecto, aunque PostgreSQL 15 (POSTGRES_15) se usa en staging de Lana Bank. En Azure, la versión por defecto es **PostgreSQL 16** (16), aunque PostgreSQL 14 también está soportado.

En GCP, se usa Cloud SQL para PostgreSQL (Enterprise Edition), mientras que en Azure se usa Azure Database for PostgreSQL Flexible Server.

### 13.2 Parámetros de Seguridad

Todas las instancias de base de datos están configuradas con **acceso solo privado**. El acceso IPv4 público está deshabilitado (`ipv4_enabled = false`) y todas las instancias están conectadas a VPC/VNet privada. SSL/TLS es requerido para todas las conexiones (`sslmode = "require"` en Azure).

Los usuarios administradores se generan automáticamente con contraseñas aleatorias de 20 caracteres. Los usuarios de aplicación se crean por base de datos con permisos específicos, y por defecto no tienen permisos para crear bases de datos (`user_can_create_db = false`).

El logging detallado es opcional (`enable_detailed_logging`). Cuando está habilitado, se configura `log_statement = "all"` (registra todas las sentencias SQL) y `log_lock_waits = "on"` (registra esperas de locks). El logging estándar está habilitado por defecto.

### 13.3 Replicación

La replicación lógica puede habilitarse vía `replication = true`. En GCP requiere `cloudsql.logical_decoding = "on"` y `cloudsql.enable_pglogical = "on"`, mientras que en Azure requiere `wal_level = "logical"`.

No es estrictamente necesario usar réplicas de lectura, pero se aconsejan en caso de que aparezcan necesidades de consulta de datos desde aplicaciones externas, con el objetivo de no sobrecargar las instancias de escritura de base de datos. En GCP, las réplicas de lectura están soportadas vía `provision_read_replica = true`, que puede ser pública o privada (`public_read_replica`). En Azure no hay configuración explícita de réplica de lectura en el código actual.

### 13.4 Backup

En GCP Cloud SQL, los backups automáticos están habilitados por defecto (`enabled = true`), junto con point-in-time recovery habilitado (`point_in_time_recovery_enabled = true`). La retención es gestionada por GCP (típicamente 7 días para backups automáticos) y la frecuencia es diaria.

En Azure PostgreSQL, los backups automáticos están habilitados con retención configurable entre 7-35 días vía `backup_retention_days`. Los backups geo-redundantes son opcionales vía `geo_redundant_backup_enabled`, y la frecuencia es gestionada por Azure.

### 13.5 Bases de Datos por Aplicación

Toda la persistencia se gestiona con PostgreSQL.

Para Lana Bank, las bases de datos principales incluyen **lana-bank** (base de datos principal de la aplicación), **meltano** (para ETL y pipelines de datos), **airflow** (para orquestación de workflows), **dagster** (para gestión de datos), y **keycloak** (para autenticación y autorización).

Cada aplicación puede tener múltiples instancias PostgreSQL: una instancia para Lana Bank, una instancia para Meltano (que puede incluir múltiples bases de datos), y una instancia para Keycloak.

---

## 14. Middleware / Integración

### 14.1 Kubernetes y Orquestación

El sistema usa Kubernetes para orquestación de contenedores. En GCP se usa versión 1.32.9-gke.1548000 (por defecto) y en Azure versión 1.30.9 (por defecto). Network Policies están habilitadas (Calico en GCP, Azure Network Policy en Azure). En GCP, Binary Authorization y Shielded Nodes (Secure Boot e Integrity Monitoring) también están habilitados. Helm versión 3.x está instalado en hosts bastion para gestión de charts.

### 14.2 Ingress y Balanceo de Carga

El sistema usa **NGINX Ingress Controller** (chart ingress-nginx versión 4.14.0 del repositorio https://kubernetes.github.io/ingress-nginx) para proporcionar controlador ingress para Kubernetes. El servicio ingress está configurado como tipo LoadBalancer, exponiendo una IP pública que recibe tráfico HTTP/HTTPS desde Internet.

### 14.3 Certificados SSL/TLS

Los certificados SSL/TLS se gestionan automáticamente por **cert-manager** (chart versión v1.19.1 del repositorio https://charts.jetstack.io), que puede usar Let's Encrypt o una CA interna según configuración.

### 14.4 Autenticación y Autorización

El sistema usa **Keycloak** (chart keycloakx versión 7.1.1 del repositorio https://codecentric.github.io/helm-charts) como servidor de identidad y acceso (IAM), con base de datos PostgreSQL dedicada.

**Oathkeeper** (chart versión 0.49.2 del repositorio https://k8s.ory.sh/helm/charts) actúa como proxy de autenticación y autorización con 2 réplicas por defecto para alta disponibilidad.

**OAuth2 Proxy** (chart versión 7.13.0 del repositorio https://oauth2-proxy.github.io/manifests) proporciona proxy de autenticación OAuth2.

### 14.5 Observabilidad y Monitoreo

**OpenTelemetry Collector** (chart versión 0.138.1 del repositorio https://open-telemetry.github.io/opentelemetry-helm-charts) recolecta métricas, logs y trazas, integrado con Honeycomb para análisis de datos.

### 14.6 Pipeline de Datos y ETL

**Dagster** (chart versión 1.12.1 del repositorio https://dagster-io.github.io/helm) orquesta pipelines de datos con base de datos PostgreSQL dedicada.

**Meltano** proporciona ETL y gestión de datos, también con base de datos PostgreSQL dedicada e integración con Airflow para orquestación.

**Airflow** orquesta workflows usando PostgreSQL (compartido con Meltano o dedicado según configuración).

### 14.7 PostgreSQL (Helm Chart)

Para PostgreSQL in-cluster (a diferencia de instancias gestionadas Cloud SQL/Azure PostgreSQL), se usa el chart PostgreSQL de Bitnami versión 16.4.13 (repositorio https://charts.bitnami.com/bitnami) con la imagen `bitnamilegacy/postgresql:14.5.0-debian-11-r35`.

### 14.8 Dependencias de Aplicación

El proyecto usa Semantic Versioning (SemVer) para todas las versiones de aplicación, chart y dependencias.

Para Lana Bank, la versión de aplicación es 0.12.3 con versión de chart 0.1.1-dev. Las dependencias incluyen PostgreSQL (Bitnami) 16.4.13, Oathkeeper 0.49.2, Keycloakx 7.1.1, Dagster 1.12.1, y OAuth2 Proxy 7.13.0.

Para Galoy Dependencies, la versión del chart es 0.10.20-dev con dependencias incluyendo cert-manager v1.19.1, ingress-nginx 4.14.0, kube-monkey 1.5.2, y opentelemetry-collector 0.138.1.

### 14.9 Configuración Recomendada

Los recursos de pod varían por componente. Los workers tienen recursos definidos en la sección 11.1.4. El Ingress Controller tiene recursos definidos en `ingress-scaling.yml`, el OpenTelemetry Collector en `otel-scaling.yml`, y Kube Monkey usa recursos mínimos (5m CPU, 25Mi memoria).

### 14.10 Arquitectura de Red (Networking)

#### 14.10.1 Topología de Red

En GCP, la red privada (VPC) usa modo de enrutamiento REGIONAL con nombre `{name_prefix}-vpc` y auto-crear subnets deshabilitado para control manual. La subred DMZ (`{name_prefix}-dmz`) usa CIDR `{network_prefix}.0.0/24` (ejemplo: 10.1.0.0/24) para hosts bastion y acceso administrativo, con Private Google Access habilitado. La subred del cluster (`{name_prefix}-cluster`) usa CIDR `{network_prefix}.0.0/17` (ejemplo: 10.1.0.0/17) para nodos de Kubernetes, con rangos de IP secundarios para pods (192.168.0.0/18) y services (192.168.64.0/18), también con Private Google Access habilitado. Opcionalmente hay una subred para Docker Host (`{name_prefix}-docker-host`) con CIDR 10.2.0.0/24 para hosts Docker de CI/CD.

En Azure, la red virtual (VNet) usa nombre `{name_prefix}-vnet` con espacio de direcciones `{network_prefix}.0.0/15` (ejemplo: 10.1.0.0/15). La subred DMZ (`{name_prefix}-dmz`) usa CIDR `{network_prefix}.0.0/24` para hosts bastion. La subred del cluster (`{name_prefix}-cluster`) aloja nodos de Kubernetes (AKS) con Service CIDR 192.168.64.0/18 y DNS Service IP 192.168.64.10. La subred PostgreSQL (`{name_prefix}-postgres`) usa CIDR `{network_prefix}.3.0/24` (ejemplo: 10.1.3.0/24) con delegación a Microsoft.DBforPostgreSQL/flexibleServers y un Network Security Group asociado con reglas para PostgreSQL (puerto 5432).

#### 14.10.2 Conectividad y NAT

En GCP, Cloud NAT está habilitado para permitir egreso a Internet desde subnets privadas. El router (nombrado `{name_prefix}-router`) usa NAT IP Allocation AUTO_ONLY y aplica a ALL_SUBNETWORKS_ALL_IP_RANGES con BGP ASN 64514. VPC Peering está configurado para servicios gestionados de GCP con un rango /16 reservado para servicios de Google vía servicenetworking.googleapis.com.

En Azure, Network Security Groups (NSG) proporcionan reglas de firewall para controlar tráfico, con PostgreSQL NSG permitiendo tráfico desde VirtualNetwork al puerto 5432. Private DNS Zones se usan para resolución de nombres de servicios gestionados, incluyendo privatelink.postgres.database.azure.com para PostgreSQL.

#### 14.10.3 Reglas de Firewall

En GCP, las reglas de firewall incluyen Intra-cluster Egress permitiendo comunicación entre pods y con el master (protocolos TCP, UDP, ICMP, SCTP, ESP, AH) a Master CIDR, subred del Cluster, rango de Pods y rango de Services. Webhook Ingress permite al master llamar webhooks en pods (puertos 8443, 443) desde Master CIDR. DMZ to Nodes permite acceso desde bastion a nodos del cluster (todos los protocolos) desde subred DMZ.

En Azure, Network Security Groups proporcionan reglas de seguridad por subred, con PostgreSQL permitiendo solo tráfico desde VirtualNetwork.

#### 14.10.4 Clusters Privados

La API de Kubernetes usa endpoints privados (no accesibles desde Internet). En GCP está configurado con `enable_private_endpoint = true` y en Azure con `private_cluster_enabled = true`. El acceso a la API está restringido a hosts bastion (subred DMZ) y redes autorizadas (master authorized networks en GCP). Los nodos no tienen IPs públicas: en GCP con `enable_private_nodes = true` y en Azure vía nodos en subred privada.

### 14.11 Acceso desde WAN y VPN

#### 14.11.1 Acceso Público a Frontends (WAN)

La arquitectura de Ingress usa NGINX Ingress Controller con tipo de servicio LoadBalancer, exponiendo una IP pública proporcionada por el proveedor cloud que recibe tráfico HTTP/HTTPS desde Internet.

El flujo de tráfico WAN sigue esta secuencia:
```
Cliente Internet → Load Balancer (IP Pública) → Ingress Controller (NGINX) → Servicios de Aplicación (según reglas de enrutamiento)
```

La configuración de Ingress incluye TLS/SSL con certificados gestionados por cert-manager (Let's Encrypt o CA interna). Los hosts configurados incluyen Portal de Clientes (ej. `app.example.com`), Panel de Administración (ej. `admin.example.com`), y Dagster (ej. `dagster.example.com`). La autenticación está integrada con OAuth2 Proxy y Oathkeeper, y rate limiting está configurado por host (solicitudes por minuto, conexiones).

La seguridad de acceso WAN incluye geo-blocking (bloqueando países no soportados configurados en NGINX), autenticación OAuth2/OIDC para acceso a paneles administrativos, TLS para todas las conexiones (HTTPS), y posibilidad de configurar WAF vía anotaciones NGINX o servicios externos.

#### 14.11.2 Acceso vía VPN (Empleados)

Existen varias opciones para acceso VPN:

**Opción 1: VPN Site-to-Site** - Configuración de VPN entre la red de oficina/corporativa y la VPC/VNet. En GCP se usa Cloud VPN o Partner VPN, y en Azure VPN Gateway (Site-to-Site). Las ventajas incluyen acceso directo a recursos internos sin exponer servicios a Internet, no se requieren IPs públicas para servicios internos, y control de acceso centralizado. Los empleados conectados a la red corporativa acceden automáticamente.

**Opción 2: VPN Client (Point-to-Site)** - Configuración de VPN cliente para acceso remoto. En GCP, Cloud VPN no soporta P2S nativamente y requiere solución de terceros. En Azure, se usa VPN Gateway (Point-to-Site) con OpenVPN o IKEv2. Las ventajas incluyen acceso desde cualquier ubicación, autenticación por certificado o usuario/contraseña, y no se requiere red corporativa. Los empleados remotos se conectan vía cliente VPN.

**Opción 3: Bastion Host con VPN** - Configuración de VPN al host bastion con port forwarding. El flujo es: el empleado se conecta a VPN, VPN termina en el host bastion, y el empleado accede a servicios internos a través del bastion. Las ventajas incluyen control de acceso granular, auditoría centralizada, y no se requieren cambios a la infraestructura principal.

**Opción 4: Acceso vía Bastion (SSH Tunneling)** - Configuración de túnel SSH a través del host bastion para acceso administrativo y debugging. Por ejemplo, tunneling a base de datos PostgreSQL vía `ssh -L localhost:5432:db-internal-ip:5432 bastion-host`.

#### 14.11.3 Recomendaciones de Seguridad

Para acceso WAN, se recomienda siempre usar HTTPS/TLS, implementar rate limiting, configurar WAF (Web Application Firewall), monitorear y alertar sobre tráfico anómalo, e implementar autenticación fuerte (2FA) para paneles administrativos.

Para acceso VPN, se recomienda usar autenticación fuerte (certificados + 2FA), implementar segmentación de red (acceso solo a recursos necesarios), monitorear conexiones VPN, rotar credenciales y certificados regularmente, y considerar Zero Trust Network Access (ZTNA) para acceso más granular.

La arquitectura híbrida recomendada expone solo servicios que requieren acceso público (portal de clientes) como frontends públicos. Backends y paneles de administración tienen acceso solo vía VPN o red privada. Las bases de datos nunca se exponen a Internet, con acceso solo desde aplicaciones dentro del cluster y administradores vía VPN + bastion.

---

## 15. Servicios Externos

La aplicación está diseñada para integrarse con varios servicios externos que proporcionan funcionalidades especializadas. Estos servicios no son parte de la infraestructura desplegada pero son componentes críticos del ecosistema operativo.

**Es importante enfatizar que estos servicios deben configurarse externamente** por el cliente o equipo de operaciones. Lana Bank simplemente espera recibir las credenciales, tokens, endpoints y otra información de configuración necesaria para integrarse con estos servicios. La aplicación no gestiona la creación, configuración o administración de cuentas en estos servicios externos; solo consume sus APIs y servicios una vez que están configurados y disponibles.

### 15.1 Sumsub para KYC/KYB

Sumsub se usa para gestionar procesos y datos de KYC (Know Your Customer) y KYB (Know Your Business). Este servicio externo maneja la verificación de identidad de clientes y empresas, incluyendo validación de documentos, verificación biométrica y cumplimiento regulatorio.

Para integrar Sumsub, es necesario configurar una cuenta en el servicio Sumsub y obtener credenciales API (API key, API secret), así como los endpoints correspondientes. Lana Bank espera recibir estas credenciales y endpoints como parte de la configuración del ambiente, y se integra con Sumsub a través de su API para enviar solicitudes de verificación y recibir resultados de procesos de onboarding y verificación continua.

### 15.2 Honeycomb para Observabilidad

Honeycomb se usa para agregación y explotación de datos OpenTelemetry, así como para generar alertas que se integran con software de gestión de pager/on-call. El sistema usa el protocolo OpenTelemetry (OTEL) para enviar métricas, logs y trazas desde el OpenTelemetry Collector a Honeycomb.

Para integrar Honeycomb, es necesario configurar una cuenta en el servicio y obtener la API key y dataset correspondiente. Lana Bank espera recibir estas credenciales como parte de la configuración del ambiente. Una vez configurado, el OpenTelemetry Collector envía automáticamente datos de telemetría al servicio.

Es importante notar que, aunque actualmente se usa Honeycomb, la aplicación usa el protocolo OTEL estándar, lo que permite migrar a otros proveedores que soporten OpenTelemetry sin modificaciones significativas a la aplicación. Proveedores alternativos compatibles incluyen Datadog, New Relic, Grafana Cloud, y otros servicios que soportan el protocolo OTEL. En todos los casos, la configuración del servicio externo (creación de cuenta, obtención de credenciales, configuración de dataset, etc.) debe hacerse externamente antes de proporcionar credenciales a Lana Bank.

Honeycomb proporciona capacidades de análisis de datos a través de consultas avanzadas, detección de anomalías y creación de dashboards personalizados. Adicionalmente, el sistema de alertas de Honeycomb se integra con sistemas de gestión pager/on-call (como PagerDuty, Opsgenie, o Zenduty) para notificar al equipo de operaciones sobre incidentes y anomalías detectadas. La configuración de estas integraciones de alertas también debe hacerse externamente en el servicio Honeycomb.

### 15.3 BigQuery para Almacenamiento de Datos de Reportes

BigQuery se usa como almacenamiento de datos analíticos y de reportes. El sistema usa BigQuery para almacenar datos transformados de las bases de datos operativas PostgreSQL, permitiendo análisis y reportes sin impactar el rendimiento de la base de datos transaccional.

La aplicación usa BigQuery en conjunto con herramientas ETL (Meltano) y transformación de datos (dbt) para cargar y transformar datos desde PostgreSQL a BigQuery. El sistema crea datasets en BigQuery para almacenar datos transformados, y usa conexiones de BigQuery a Cloud SQL para leer datos directamente desde PostgreSQL cuando es necesario.

Para integrar BigQuery, es necesario configurar el servicio en GCP (creación de dataset, configuración de permisos, creación de service account, etc.) y proporcionar a Lana Bank las credenciales necesarias, incluyendo el JSON de service account, project ID, y nombres de datasets. La aplicación espera recibir estas credenciales como parte de la configuración del ambiente.

**Es importante notar que, aunque actualmente se usa BigQuery, la aplicación puede refactorizarse para realizar el mismo trabajo en otras bases de datos analíticas.** El código ETL y de transformación puede adaptarse para trabajar con alternativas como Amazon Redshift, Snowflake, Azure Synapse Analytics, o incluso bases de datos analíticas on-premise. La arquitectura de datos está diseñada para que la capa de almacenamiento analítico pueda intercambiarse sin afectar significativamente la lógica de negocio, aunque requerirá trabajo de desarrollo para adaptar conectores y transformaciones a la nueva plataforma elegida.

En ambientes Azure, donde BigQuery no está disponible, alternativas nativas como Azure Synapse Analytics o Azure Data Factory pueden usarse para realizar funciones similares de almacenamiento y procesamiento analítico.

### 15.4 Notas Adicionales

#### 15.4.1 Configuración por Ambiente

Diferentes ambientes tendrán diferentes necesidades y tendrán que ajustarse a la misma arquitectura base. Los valores de recursos, conteos de réplicas y configuraciones específicas pueden variar según las necesidades de cada ambiente.

#### 15.4.2 Actualizaciones

Las versiones de Kubernetes se actualizan manualmente. Las versiones de aplicación y charts se gestionan vía vendir y referencias a repositorios externos. Las actualizaciones de base de datos deben planificarse cuidadosamente debido a posible downtime. Todas las versiones siguen Semantic Versioning (SemVer).
