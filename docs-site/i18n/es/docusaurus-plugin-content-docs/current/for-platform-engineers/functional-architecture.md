---
sidebar_position: 1
---

# Arquitectura Funcional de Lana Bank

## Tabla de Contenidos

1. [Descripción General](#1-descripción-general)
2. [Arquitectura de la Aplicación](#2-arquitectura-de-la-aplicación)
3. [Flujos de Comunicación](#3-flujos-de-comunicación)
4. [Integraciones con Sistemas Externos](#4-integraciones-con-sistemas-externos)
5. [Flujos de Autenticación y Seguridad](#5-flujos-de-autenticación-y-seguridad)
6. [Segmentación de Red por Entorno](#6-segmentación-de-red-por-entorno)
7. [Zonas de Seguridad](#7-zonas-de-seguridad)
8. [Auditoría](#8-auditoría)
9. [Flujo de Préstamos Respaldados por Bitcoin](#9-flujo-de-préstamos-respaldados-por-bitcoin)
10. [Portabilidad y Dependencia de Proveedor](#10-portabilidad-y-dependencia-de-proveedor)
11. [Servidores / Instancias](#11-servidores--instancias)
12. [Sistemas Operativos](#12-sistemas-operativos)
13. [Bases de Datos](#13-bases-de-datos)
14. [Middleware / Integración](#14-middleware--integración)
15. [Servicios Externos](#15-servicios-externos)

---

## 1. Descripción General

Este documento describe la arquitectura lógica de Lana Bank, incluyendo la arquitectura interna de la aplicación, integraciones con sistemas externos, flujos de autenticación y seguridad, segmentación de red por entorno y zonas de seguridad.

### 1.1 Visión General

Lana Bank es una aplicación de core bancario especializada en **préstamos respaldados por Bitcoin**. La arquitectura sigue los principios de **Domain-Driven Design (DDD)** y **Arquitectura Hexagonal**, separando claramente las capas de dominio, aplicación e infraestructura.

El backend está desarrollado en Rust, utilizando PostgreSQL como base de datos principal y **Cala Ledger** como motor contable de doble entrada con garantías de consistencia fuerte. Los frontends web están construidos con Next.js y TypeScript, consumiendo APIs GraphQL expuestas por el backend. Para reportes y análisis, existe un pipeline de datos basado en Meltano que extrae información hacia BigQuery, donde los datos se transforman con dbt.

---

## 2. Arquitectura de la Aplicación

### 2.1 Módulos del Core Bancario

Los módulos principales implementan la lógica de negocio del banco, siguiendo principios de Event Sourcing donde cada entidad mantiene su estado como una secuencia de eventos inmutables.

#### 2.1.1 Crédito

El módulo de crédito es el corazón del sistema, gestionando el ciclo de vida completo de los préstamos respaldados por Bitcoin. Una línea de crédito atraviesa un ciclo de vida bien definido que comienza cuando un operador crea una **CreditFacilityProposal** para un cliente. Esta propuesta entra automáticamente en un proceso de aprobación gestionado por el módulo de gobernanza; los miembros del comité asignado deben votar para aprobarla.

Una vez aprobada, la propuesta se transforma en una **PendingCreditFacility**. En esta etapa, el cliente debe depositar la garantía en Bitcoin requerida. Si la línea tiene un custodio asignado, los webhooks del custodio mantienen automáticamente sincronizado el saldo de la garantía. Si no hay custodio (modo manual), un operador puede actualizar la garantía directamente. El sistema monitorea continuamente el ratio de colateralización (CVL - Collateral Value to Loan) comparándolo con el precio actual de Bitcoin.

La línea se activa automáticamente cuando el CVL alcanza el umbral inicial configurado en los términos. Los **TermValues** definen todos los parámetros del préstamo: la tasa de interés anual, la duración (clasificada como corto o largo plazo dependiendo de si excede los 12 meses), los intervalos de acumulación de intereses (diarios o mensuales), la comisión inicial (tarifa única), y tres umbrales críticos de CVL que deben mantener una jerarquía estricta: el CVL inicial debe ser mayor que el CVL de margin call, que a su vez debe ser mayor que el CVL de liquidación. También se configura la política de desembolso, que puede ser única o múltiple.

Con la **CreditFacility** activa, el cliente puede solicitar **Disbursals**. Cada desembolso pasa por su propio proceso de aprobación. Cuando se ejecuta, los fondos se acreditan en la cuenta de depósito del cliente y se crea una **Obligation** que representa la deuda. Las obligaciones tienen un ciclo de estados: comienzan como "aún no vencidas", pasan a "vencidas" en la fecha de vencimiento, pueden volverse "morosas" si no se pagan a tiempo, entrar en "liquidación" si la morosidad persiste, y finalmente ser marcadas como "en default".

El sistema ejecuta trabajos periódicos para la acumulación de intereses. Los **InterestAccrualCycles** calculan los intereses según los intervalos configurados y generan nuevas obligaciones por los intereses acumulados. Cuando el cliente realiza un **Payment**, el sistema asigna automáticamente los fondos a las obligaciones pendientes en orden de prioridad a través de **PaymentAllocation**, típicamente priorizando las obligaciones más antiguas y los intereses sobre el principal.

Si el CVL cae por debajo del umbral de margin call, la línea entra en estado de alerta. Si cae por debajo del umbral de liquidación, se inicia un **LiquidationProcess** donde el banco puede ejecutar la garantía para recuperar la deuda. El sistema implementa un búfer de histéresis para evitar oscilaciones frecuentes entre estados cuando el CVL está cerca de los umbrales.

#### 2.1.2 Depósitos

El módulo de depósitos gestiona las cuentas donde los clientes mantienen sus fondos en USD. Cuando se crea una **CuentaDeDepósito** para un cliente, el sistema genera automáticamente las cuentas contables correspondientes en el libro mayor. La categorización contable depende del tipo de cliente: las cuentas para individuos, entidades gubernamentales, empresas privadas, bancos, instituciones financieras y empresas no domiciliadas se agrupan bajo diferentes nodos del plan de cuentas.

Los **Depósitos** representan entradas de fondos a la cuenta y se registran inmediatamente. Los **Retiros** siguen un flujo más controlado: cuando se inician, los fondos se reservan en contabilidad y se crea un proceso de aprobación. El comité asignado debe aprobar el retiro antes de que se ejecute. Si se aprueba, los fondos salen de la cuenta; si se rechaza o cancela, la reserva se revierte. También existe la posibilidad de revertir depósitos ya registrados cuando sea necesario.

Las cuentas pueden estar en diferentes estados que afectan las operaciones permitidas. Una cuenta **activa** permite todas las operaciones normales. Una cuenta **congelada** impide nuevas operaciones pero mantiene el saldo visible; esto es útil para situaciones de cumplimiento donde los fondos necesitan ser bloqueados temporalmente. Una cuenta **cerrada** es permanente y solo se permite si el saldo es cero. El módulo también admite la actualización masiva del estado de todas las cuentas de un cliente, por ejemplo cuando cambia su verificación KYC.

El historial de la cuenta puede consultarse a través del libro mayor, mostrando todas las transacciones que han afectado el saldo. El módulo calcula el saldo disponible considerando los retiros pendientes de aprobación.

#### 2.1.3 Clientes

Este módulo gestiona la información sobre los clientes del banco y es fundamental para el cumplimiento normativo. Cada cliente se crea con un tipo específico que determina su tratamiento contable y regulatorio: **Individual** para personas naturales, **EntidadGubernamental** para entidades gubernamentales, **EmpresaPrivada** para empresas privadas, **Banco** para bancos, **InstituciónFinanciera** para otras instituciones financieras, **AgenciaOSucursalExtranjera** para agencias extranjeras, y **EmpresaNoDomiciliada** para empresas no domiciliadas.

El proceso de verificación KYC se integra con SumSub. Un cliente comienza en estado **VerificaciónPendiente**. Cuando SumSub notifica mediante webhook que la verificación fue exitosa, el cliente pasa a **Verificado** con un nivel KYC (Básico o Avanzado). Si la verificación falla, permanece en estado **Rechazado**. El sistema puede configurarse para requerir verificación antes de permitir la creación de cuentas de depósito o facilidades crediticias.

El módulo gestiona los documentos asociados al cliente, almacenándolos en la nube y permitiendo la generación de enlaces de descarga temporales. Los documentos pueden archivarse o eliminarse según sea necesario.

Para el cumplimiento de las regulaciones sobre cuentas inactivas, el sistema rastrea la última actividad de cada cuenta de depósito. Un trabajo periódico clasifica automáticamente las cuentas de depósito según su actividad: **Activa** si ha tenido actividad reciente (menos de un año), **Inactiva** si ha estado entre uno y diez años sin actividad, y **Escheatable** si supera los diez años. Esta clasificación es independiente del estado operativo de la cuenta y no congela ni cierra la cuenta por sí misma.

#### 2.1.4 Custodia

El módulo de custodia proporciona una abstracción sobre múltiples proveedores de custodia de Bitcoin, permitiendo al banco trabajar con diferentes custodios según sus necesidades operativas y regulatorias. El sistema está diseñado con un patrón de complementos donde cada **Custodio** implementa una interfaz común. Se han implementado **BitGo**, **Komainu** y una opción de **Autocustodia** basada en xpub, y la arquitectura permite agregar nuevos custodios sin modificar el resto del sistema.

En cada implementación, se pueden configurar y activar múltiples custodios simultáneamente. Al crear una línea de crédito, puede especificar qué custodio gestionará la garantía de esa línea en particular. Esto permite, por ejemplo, utilizar diferentes custodios para distintos segmentos de clientes o jurisdicciones.

Cada custodio gestiona **Carteras** que se asignan a líneas de crédito para recibir garantías en Bitcoin. Los custodios alojados notifican al sistema sobre cambios en los saldos de las carteras mediante webhooks. La opción de autocustodia almacena únicamente una `xpub` de cuenta en el backend, deriva una nueva dirección de recepción para cada préstamo recién creado y consulta un backend esplora para detectar cambios de saldo confirmados. La URL base de esplora se selecciona al inicio desde `lana.yml` según la red de autocustodia configurada. El flujo de autocustodia admite configuraciones de cuenta para mainnet, testnet3, testnet4 y signet. En ambos casos, Lana actualiza la **Garantía** asociada con la línea correspondiente y recalcula el CVL automáticamente.

Los webhooks de los custodios se reciben en endpoints específicos del proveedor y se validan criptográficamente antes de procesarse. La configuración para custodios alojados incluye las credenciales de API y claves necesarias para verificar la autenticidad de los webhooks, y esos valores sensibles se almacenan cifrados. Para la autocustodia, el backend conserva únicamente la `xpub`; la `xpriv` correspondiente se genera y retiene fuera del backend, mientras que las URL de esplora específicas de cada red se proporcionan mediante la configuración de inicio.

#### 2.1.5 Contabilidad

El módulo de contabilidad implementa un sistema completo de contabilidad por partida doble, fundamental para cualquier institución financiera regulada. Utiliza **Cala Ledger** como motor subyacente, un crate especializado de Rust que proporciona plantillas de transacciones predefinidas y garantías de consistencia ACID para todas las operaciones contables.

El **ChartOfAccounts** define la estructura jerárquica de cuentas del banco. Puede importarse desde archivos CSV y soporta una estructura de árbol con múltiples niveles. Cada nodo del árbol puede ser una cuenta individual o un grupo que agrega las cuentas de sus hijos. El plan de cuentas se integra con otros módulos: las cuentas de depósito de clientes, las facilidades crediticias y las cuentas de garantía se crean automáticamente como hijos de los nodos apropiados según el tipo de cliente y producto.

Cada **LedgerAccount** tiene un tipo de saldo normal (débito o crédito) y puede mantener saldos en múltiples monedas (USD y BTC). Las **LedgerTransactions** representan movimientos contables que siempre mantienen el equilibrio: el total de débitos es igual al total de créditos. El sistema registra automáticamente transacciones para cada operación del negocio: depósitos, retiros, desembolsos, pagos de préstamos, acumulación de intereses y actualizaciones de garantías.

Para los informes financieros, el módulo genera el **TrialBalance** que enumera todas las cuentas con sus saldos deudores y acreedores, útil para verificar que los libros estén cuadrados. El **BalanceSheet** presenta la posición financiera del banco organizando activos, pasivos y patrimonio. El **ProfitAndLoss** muestra los ingresos (principalmente intereses de préstamos) menos los gastos para calcular el resultado del período.

El sistema soporta múltiples **FiscalYears** y permite consultar saldos e informes para rangos de fechas específicos. También permite **ManualTransactions** para ajustes contables que no se originan de operaciones automatizadas del sistema.

#### 2.1.6 Gobernanza

El sistema de gobernanza proporciona un marco flexible para implementar flujos de aprobación multifirma sobre operaciones sensibles. Está diseñado para adaptarse a diferentes estructuras organizacionales y requisitos regulatorios.

Los **Committees** representan grupos de personas autorizadas para tomar decisiones sobre ciertos tipos de operaciones. Un comité puede tener cualquier número de miembros, típicamente usuarios del sistema con roles específicos. El mismo usuario puede pertenecer a múltiples comités.

Las **Policies** definen las reglas de aprobación para cada tipo de proceso. Una política especifica qué comité es responsable de aprobar ese tipo de operación y cuál es el umbral requerido: el número mínimo de votos afirmativos necesarios para aprobar. Por ejemplo, una política para aprobación de desembolsos podría requerir 2 de 3 miembros del comité de crédito.

Cuando se inicia una operación que requiere aprobación, el sistema crea automáticamente un **ApprovalProcess** vinculado a la política correspondiente. El proceso comienza en estado pendiente y registra los votos de los miembros del comité. Un miembro puede votar a favor o en contra (con un motivo obligatorio). Cuando se alcanza el umbral de aprobación, el proceso se marca como aprobado y se emite un evento **ApprovalProcessConcluded**. Si algún miembro vota en contra, el proceso termina inmediatamente como rechazado.

Los eventos de conclusión de procesos de aprobación son consumidos por trabajos que ejecutan la operación aprobada o manejan el rechazo. Este diseño desacopla el flujo de aprobación de la ejecución, permitiendo que las aprobaciones se procesen de forma asíncrona.

#### 2.1.7 Acceso

El módulo de acceso implementa control de acceso basado en roles (RBAC) para todos los operadores del sistema. Los **Usuarios** representan a las personas que operan el banco a través del Panel de Administración. Cada usuario tiene un identificador único que se vincula con el sistema de autenticación externo.

Los **Roles** agrupan conjuntos de permisos y se asignan a los usuarios. Un usuario puede tener múltiples roles, y sus permisos efectivos son la unión de los permisos de todos sus roles. Los **PermissionSets** son colecciones nombradas de permisos específicos que facilitan la configuración de roles comunes.

El sistema de permisos es granular: cada operación en cada módulo tiene un permiso asociado. Por ejemplo, existen permisos separados para leer clientes, crear clientes, aprobar KYC, visualizar facilidades crediticias, iniciar desembolsos, etc. Antes de ejecutar cualquier operación, el sistema verifica que el usuario tenga el permiso correspondiente y registra la acción en el registro de auditoría.

El sistema de autorización utiliza **Casbin**, un motor de control de acceso flexible, con políticas almacenadas en PostgreSQL para persistencia y sincronización entre instancias. El modelo RBAC sigue una estructura de tres niveles: Usuario → Rol → PermissionSet → Permisos (Objeto + Acción).

Cada módulo define sus propios conjuntos de permisos que agrupan acciones relacionadas. Los conjuntos de permisos típicos siguen un patrón de visualizador/escritor. El sistema incluye roles predefinidos como Admin (acceso completo), Bank Manager (similar a admin pero sin acceso a la gestión de acceso ni custodia), y Accountant (enfocado en funciones de contabilidad y visualización).

Los permisos se gestionan dinámicamente a través de la API y los cambios persisten inmediatamente en la base de datos, recargándose en cada verificación de permisos, lo que garantiza que las actualizaciones sean efectivas sin necesidad de reiniciar el sistema.

#### 2.1.8 Precio

Este módulo obtiene y gestiona los precios de Bitcoin, una función crítica para un banco que ofrece préstamos con garantía de BTC. El sistema se integra con Bitfinex para obtener precios en tiempo real a través de su API.

Cuando se obtiene un nuevo precio, el módulo publica un **CorePriceEvent** que otros módulos consumen. El módulo de crédito es el principal consumidor: utiliza el precio para calcular el CVL de todas las facilidades activas y determinar si alguna ha caído por debajo de los umbrales de margin call o liquidación. Los cambios de precio pueden desencadenar actualizaciones de estado en las facilidades y potencialmente iniciar procesos de liquidación.

#### 2.1.9 Informes

El módulo de informes coordina la generación de reportes regulatorios y operativos. Define tipos de **Report** que especifican qué datos incluir y en qué formato. Cada ejecución de informe se registra como un **ReportRun** con su estado (pendiente, ejecutando, completado, fallido) y archivos generados.

La generación de informes se integra con el pipeline de datos: los datos transformados en BigQuery alimentan los informes finales. El sistema puede integrarse con sistemas de reporting externos según las necesidades regulatorias de cada jurisdicción donde opera el banco.

#### 2.1.10 Módulos de Soporte

Además de los módulos principales, existen módulos de soporte: **document-storage** para el almacenamiento de documentos en la nube, **public-id** para generar identificadores públicos legibles para las entidades, y **core-money** que define primitivos monetarios (UsdCents, Satoshis) utilizados en todo el sistema.

### 2.2 Capa de Aplicación

El directorio `lana/` contiene la capa de aplicación que orquesta los módulos principales y expone la funcionalidad externamente.

#### 2.2.1 Servidores GraphQL

El sistema expone dos servidores GraphQL independientes. El **admin-server** sirve al panel de administración utilizado por los operadores del banco, mientras que el **customer-server** sirve al portal de clientes. Ambos servidores incluyen playground integrado para desarrollo y reciben webhooks de servicios externos.

#### 2.2.2 Servicios de Aplicación

El servicio principal **lana-app** orquesta la inicialización de todos los módulos y proporciona el punto de entrada unificado. **lana-cli** ofrece una interfaz de línea de comandos para operaciones administrativas.

Existen servicios especializados para diferentes funciones: **notification** gestiona el envío de correos electrónicos, **contract-creation** genera contratos PDF, **customer-sync** y **deposit-sync** sincronizan datos con sistemas externos, **user-onboarding** administra el registro de operadores, y **dashboard** calcula métricas agregadas. Para desarrollo y pruebas, **sim-bootstrap** permite inicializar datos de simulación.

#### 2.2.3 Sistema de Eventos

El módulo **lana-events** define el enum unificado **LanaEvent** que agrupa todos los eventos de dominio del sistema, permitiendo que el sistema de outbox y los trabajos procesen eventos de cualquier módulo de manera uniforme.

### 2.3 Interfaces Web

#### 2.3.1 Panel de Administración

El Panel de Administración es la interfaz principal para operadores y personal del banco. Permite gestionar clientes y sus procesos KYC, administrar facilidades crediticias en todas las etapas, aprobar desembolsos y retiros, y gestionar cuentas de depósito. También proporciona acceso a la visualización contable completa (balance, estado de resultados, balance de comprobación), configuración de comités y políticas de aprobación, gestión de usuarios y roles, y generación de reportes regulatorios.

#### 2.3.2 Portal del Cliente

El Portal del Cliente está orientado hacia los clientes del banco. Actualmente ofrece funcionalidad de solo lectura, permitiendo visualizar facilidades crediticias, estado de desembolsos e historial de transacciones. La arquitectura permite extenderlo en el futuro para soportar operaciones desde el lado del cliente.

#### 2.3.3 Web Compartida

El módulo **shared-web** contiene componentes de interfaz compartidos entre ambos portales, asegurando consistencia visual y reduciendo la duplicación de código.

---

## 3. Flujos de Comunicación

### 3.1 Event Sourcing y Eventos de Dominio

El sistema utiliza **Event Sourcing** como patrón arquitectónico central. Cada entidad recibe comandos que generan eventos, estos eventos se persisten en la base de datos como única fuente de verdad, y el estado actual de la entidad se reconstruye aplicando la secuencia de eventos.

Este diseño proporciona auditabilidad completa (cada cambio queda registrado), la capacidad de reconstruir el estado en cualquier momento, y la posibilidad de agregar nuevas proyecciones sobre datos históricos.

La comunicación entre módulos ocurre a través de eventos públicos. Cada módulo define sus propios eventos en un enum específico (por ejemplo, **CoreCreditEvent** para el módulo de crédito). Un **Publisher** asociado a cada módulo transforma los eventos internos de entidades en eventos públicos que otros módulos pueden consumir.

Los eventos públicos típicos incluyen: creación y aprobación de propuestas de crédito, activación y finalización de facilidades, cambios de colateralización, desembolsos liquidados, devengo de intereses, creación y transición de obligaciones entre estados (vencida, en mora, incobrable), pagos registrados y procesos de liquidación. Cada evento incluye marcas de tiempo de cuándo fue registrado y cuándo fue efectivo, permitiendo reconstrucciones precisas del estado en cualquier momento.

### 3.2 Patrón Outbox

Para integraciones con sistemas externos que requieren garantías de entrega, el sistema implementa el **Patrón Outbox**. Cuando un módulo necesita publicar un evento, lo persiste en una tabla outbox dentro de la misma transacción de base de datos que la operación de negocio. Esto garantiza atomicidad: o ambos (la operación y el evento) persisten, o ninguno.

PostgreSQL NOTIFY informa inmediatamente a los listeners cuando hay nuevos eventos, evitando la necesidad de polling.

El sistema soporta dos tipos de eventos en el outbox. Los **eventos persistentes** tienen un identificador único, un número de secuencia global monotónicamente creciente, el payload serializado como JSON, contexto de trazabilidad para correlación distribuida y marca de tiempo de cuándo fue registrado. Los **eventos efímeros** no tienen secuencia y se utilizan para notificaciones en tiempo real que no necesitan durabilidad.

Este diseño garantiza **entrega al menos una vez**: un sistema externo puede consumir eventos con certeza de que no perderá ninguno, aunque podría recibir duplicados que debe manejar de forma idempotente.

### 3.3 Sistema de Trabajos Asíncronos

Las operaciones que no deben bloquear el flujo principal se ejecutan a través de un sistema de trabajos asíncronos. Los workers se ejecutan como procesos separados del servidor principal, permitiendo escalar el procesamiento de forma independiente de los servidores de API.

Los trabajos pueden programarse de varias formas: ejecutar inmediatamente, programar para una fecha/hora futura específica, o reprogramar al completarse para ejecutarse nuevamente. Esta flexibilidad es esencial para los flujos temporales del sistema bancario. Por ejemplo, cuando se crea una obligación, se programa un trabajo para la fecha de vencimiento. Cuando ese trabajo se ejecuta, si la obligación no está pagada, la marca como "vencida" y programa el siguiente trabajo para la fecha de morosidad. La cadena continúa: vencida → morosa → liquidación → incumplida, cada transición programada con precisión según los términos de la facilidad.

Para el devengo de intereses, un trabajo procesa cada devengo diario y se reprograma automáticamente para el día siguiente. Cuando un período de devengo termina (típicamente a fin de mes), programa un trabajo de ciclo de devengo que consolida los intereses y crea la obligación correspondiente.

Otros trabajos procesan flujos de eventos del outbox de forma continua, manteniendo su estado de ejecución (el último evento procesado) y reprogramándose inmediatamente cuando no hay nuevos eventos para continuar escuchando.

### 3.4 Webhooks Entrantes

Los servicios externos notifican al sistema a través de webhooks. **SumSub** envía notificaciones sobre el ciclo de vida de verificación KYC a `/webhook/sumsub`. Cuando un cliente completa su verificación, SumSub notifica el resultado (aprobado o rechazado). El sistema procesa esta notificación y actualiza el estado KYC del cliente, lo cual puede desbloquear la creación de cuentas de depósito o líneas de crédito según la configuración.

Los **custodios de Bitcoin** (BitGo, Komainu) notifican eventos de billetera a `/webhook/custodian/[provider]`. Cada proveedor tiene su propio formato de webhook que el sistema normaliza. Los eventos típicos incluyen depósitos de Bitcoin en billeteras de colateral. Cuando llega una notificación, el sistema verifica su autenticidad (típicamente mediante HMAC), identifica la billetera afectada, actualiza el saldo de colateral correspondiente y recalcula el CVL de la línea de crédito asociada. Si el nuevo CVL cruza algún umbral configurado, se actualiza el estado de colateralización y se publican los eventos correspondientes.

Este flujo de webhooks es crítico para la gestión de riesgo en tiempo real. Sin él, el sistema dependería de sondeos periódicos y podría tener visibilidad tardía de los cambios en el colateral, aumentando el riesgo durante caídas en el precio de Bitcoin.

### 3.5 Flujo de API GraphQL

Las solicitudes del cliente web siguen este flujo: el cliente envía una solicitud GraphQL con un token JWT. El middleware extrae el sujeto del token y lo inyecta en el contexto. El resolver invoca el caso de uso correspondiente en lana-app, que primero verifica los permisos RBAC y luego ejecuta la operación en el módulo central apropiado. Los eventos generados se publican y la respuesta retorna al cliente.

---

## 4. Integraciones con Sistemas Externos

La aplicación está diseñada para integrarse con diversos servicios externos que proveen funcionalidades especializadas. Estos servicios no forman parte de la infraestructura desplegada, pero son componentes críticos del ecosistema operativo.

**Es importante enfatizar que estos servicios deben configurarse externamente** por parte del cliente o del equipo de operaciones. La aplicación simplemente espera recibir las credenciales, tokens, endpoints y demás información de configuración necesaria para integrarse con estos servicios. La aplicación no gestiona la creación, configuración o administración de cuentas en estos servicios externos; únicamente consume sus APIs y servicios una vez que están configurados y disponibles.

### 4.1 KYC/KYB y AML (Conozca a su Cliente / Conozca su Negocio / Anti-Lavado de Dinero)

#### 4.1.1 Sumsub

Sumsub se utiliza para gestionar los procesos y datos de KYC (Conozca a su Cliente) y KYB (Conozca su Negocio). Este servicio externo maneja la verificación de identidad de clientes y empresas, incluyendo:

- Validación de documentos de identidad
- Verificación biométrica
- Verificación de documentos corporativos
- Cumplimiento normativo
- Incorporación de clientes y empresas
- Verificación continua

Sumsub también satisface las necesidades de AML (Anti-Lavado de Dinero) además de proporcionar capacidades de KYC/KYB. Sumsub incluye funcionalidades de detección y prevención de lavado de dinero, tales como:

- Verificación de listas de sanciones (OFAC, ONU, etc.)
- Análisis de transacciones sospechosas
- Monitoreo de patrones de comportamiento
- Informes regulatorios automáticos
- Integración con sistemas de cumplimiento

La aplicación se integra con Sumsub a través de su API REST. Para configurar la integración, es necesario configurar una cuenta en el servicio de Sumsub, obtener las credenciales de la API (clave de API, secreto de API), configurar los endpoints correspondientes (pueden variar según la región) y proporcionar estas credenciales y endpoints como parte de la configuración del entorno.

El flujo de integración funciona de la siguiente manera: la aplicación envía solicitudes de verificación a Sumsub a través de su API. Sumsub procesa las solicitudes y realiza las verificaciones necesarias. Los resultados de los procesos de incorporación y verificación continua se reciben mediante webhooks en el endpoint `/webhook/sumsub`. Cuando un cliente completa su verificación, SumSub notifica el resultado (aprobado o rechazado), y el sistema procesa esta notificación actualizando el estado KYC del cliente, lo cual puede desbloquear la creación de cuentas de depósito o líneas de crédito según la configuración.

La arquitectura también está preparada para integrar sistemas AML adicionales si fuera necesario. Las integraciones AML suelen incluir las funcionalidades mencionadas anteriormente. La aplicación puede integrarse con proveedores de servicios AML a través de APIs REST o mediante la integración con sistemas de terceros. La configuración seguiría el mismo patrón que otras integraciones externas: las credenciales y endpoints se proporcionan como parte de la configuración del entorno.

### 4.2 Pasarelas de Pago

**Nota importante:** Las integraciones con pasarelas de pago no están implementadas en la versión actual de Lana. Sin embargo, debido a que Lana tiene un diseño modular, la arquitectura prevé que estos elementos se añadirán eventualmente según las necesidades del negocio.

La aplicación está diseñada para integrarse con pasarelas de pago externas para procesar transacciones financieras. Aunque las pasarelas específicas pueden variar según el cliente y la región, la arquitectura admite la integración con múltiples proveedores.

La aplicación está diseñada para admitir varios tipos de integración:

- Procesamiento de pagos con tarjeta (débito/crédito)
- Transferencias bancarias (ACH, transferencias electrónicas, etc.)
- Procesamiento de pagos móviles
- Integración con sistemas de compensación y liquidación

Las pasarelas de pago se integrarían mediante APIs REST o SOAP. Las credenciales de API, endpoints y configuraciones específicas se proporcionarían como parte de la configuración del entorno. La aplicación está diseñada para admitir múltiples pasarelas simultáneamente, permitiendo el enrutamiento de transacciones según las reglas de negocio.

Todas las comunicaciones con las pasarelas de pago utilizarían TLS/SSL para el cifrado en tránsito. Las credenciales sensibles se almacenarían como secretos en Kubernetes y se inyectarían en los contenedores de la aplicación mediante variables de entorno o volúmenes montados.

### 4.3 BCR (Banco Central de Reserva)

**Nota importante:** La integración con el Banco Central de Reserva (BCR) no está implementada en la versión actual de Lana. Sin embargo, debido a que Lana tiene un diseño modular, la arquitectura prevé que esta integración se añadirá eventualmente según las necesidades del negocio.

La aplicación está diseñada para incluir soporte para operaciones con el Banco Central de Reserva (BCR), que es el banco central de El Salvador. Esta integración sería crítica para las operaciones bancarias regulatorias.

El sistema está diseñado para admitir varios tipos de operaciones con el BCR:

- Depósitos en el BCR (moneda local y extranjera)
- Operaciones de reporto con el BCR
- Operaciones de financiamiento con el BCR
- Reportes regulatorios y cumplimiento
- Operaciones de liquidez

La integración con el BCR se realizaría a través de sistemas estándar de comunicación bancaria (típicamente SWIFT, sistemas de mensajería financiera o APIs específicas del BCR). La configuración incluiría credenciales de acceso a los sistemas del BCR, endpoints de comunicación, certificados digitales para autenticación y configuración de formatos de mensajes (ISO 20022, formatos propietarios, etc.).

Las operaciones del BCR se procesarían a través de workers dedicados que manejarían la comunicación asíncrona y el procesamiento de respuestas. Los datos de las operaciones se registrarían en la base de datos principal y se integrarían con el sistema contable.

### 4.4 Fuentes de Datos Regulatorias

**Nota importante:** Las integraciones con fuentes de datos regulatorias no están implementadas en la versión actual de Lana. Sin embargo, debido a que Lana tiene un diseño modular, la arquitectura anticipa que estos elementos eventualmente se añadirán según las necesidades del negocio.

La aplicación está diseñada para integrarse con múltiples fuentes de datos regulatorias para cumplimiento y reportes. Estas incluirían:

- Sistemas de reporte del banco central
- Sistemas de información crediticia
- Registros públicos (registro mercantil, registro de la propiedad, etc.)
- Sistemas gubernamentales de verificación de identidad
- Sistemas de intercambio de información financiera

Las integraciones con fuentes de datos regulatorias se realizarían mediante:

- APIs REST o SOAP proporcionadas por organismos reguladores
- Sistemas de mensajería financiera (SWIFT, sistemas propietarios)
- Archivos por lotes para intercambio de datos
- Portales web con autenticación y extracción automatizada (cuando sea necesario)

Los workers de la aplicación procesarían las integraciones con sistemas regulatorios de forma asíncrona. Los datos recibidos serían validados, transformados y almacenados en la base de datos. Los reportes regulatorios se generarían automáticamente según los requisitos y se enviarían a través de los canales apropiados.

### 4.5 Observabilidad

#### 4.5.1 Honeycomb

Honeycomb se utiliza para la agregación y explotación de datos de OpenTelemetry, así como para la generación de alertas que se integran con software de gestión de guardias/guardia de llamadas. El sistema utiliza el protocolo OpenTelemetry (OTEL) para enviar métricas, registros y trazas desde el OpenTelemetry Collector a Honeycomb.

El OpenTelemetry Collector está configurado con la clave API y el conjunto de datos de Honeycomb. Los datos se envían automáticamente mediante el protocolo OTEL. Aunque actualmente se utiliza Honeycomb, la aplicación usa el protocolo estándar OTEL, lo que permite migrar a otros proveedores compatibles (Datadog, New Relic, Grafana Cloud, etc.) sin modificaciones significativas.

El sistema está instrumentado para proporcionar visibilidad completa de su comportamiento en producción. OpenTelemetry captura trazas de todas las operaciones, desde la recepción de una solicitud HTTP hasta la respuesta final. Cada operación significativa crea un span con atributos relevantes. Los spans se propagan a través de llamadas asíncronas y entre servicios, permitiendo reconstruir el flujo completo de una operación.

Las trazas se exportan a Honeycomb, donde pueden analizarse para identificar cuellos de botella, errores y patrones de uso. La propagación del contexto de rastreo a través del outbox permite correlacionar la operación original con su procesamiento asíncrono posterior.

El registro de eventos utiliza el crate **tracing** de Rust, que proporciona registros estructurados con niveles (error, warn, info, debug, trace) y campos tipados. Los registros se emiten en formato JSON en producción, facilitando su indexación y búsqueda. Cada entrada de registro incluye automáticamente el contexto del span actual, conectándola con la traza distribuida.

### 4.6 Almacenamiento de Datos de Reportes

#### 4.6.1 BigQuery

BigQuery se utiliza como almacenamiento de datos analíticos y de reportes. El sistema usa BigQuery para almacenar datos transformados de las bases de datos operacionales de PostgreSQL, permitiendo análisis y generación de reportes sin afectar el rendimiento de la base de datos transaccional.

La aplicación utiliza BigQuery en conjunto con herramientas ETL (Meltano) y de transformación de datos (dbt) para cargar y transformar datos desde PostgreSQL hacia BigQuery. Meltano extrae datos de múltiples fuentes: el extractor principal **tap-postgres** obtiene eventos y entidades del núcleo bancario, y extractores adicionales obtienen precios históricos de Bitfinex y datos de verificación KYC de SumSub.

Los datos se cargan en BigQuery, donde dbt los transforma a través de capas: staging (limpieza de datos sin procesar), intermediate (lógica de negocio) y outputs (reportes finales). El sistema genera reportes regulatorios que pueden integrarse con sistemas externos según las necesidades de cada jurisdicción.

La configuración incluye el JSON de la cuenta de servicio, el ID del proyecto y los nombres de los conjuntos de datos. **Es importante destacar que, aunque actualmente se utiliza BigQuery, la aplicación puede ser refactorizada para realizar el mismo trabajo en otras bases de datos analíticas.** El código de ETL y transformación puede adaptarse para trabajar con alternativas como Amazon Redshift, Snowflake, Azure Synapse Analytics, o incluso bases de datos analíticas on-premise.

---

## 5. Flujos de Autenticación y Seguridad

### 5.1 IAM (Gestión de Identidades y Accesos)

#### 5.1.1 Keycloak

Keycloak actúa como el servidor central de identidad y acceso (IAM) integrado con la aplicación. Proporciona:

- Gestión de usuarios y roles
- Autenticación mediante múltiples métodos (usuario/contraseña, OAuth2, OIDC)
- Autorización basada en roles (RBAC)
- Inicio de sesión único (SSO)
- Gestión de sesiones
- Integración con proveedores de identidad externos (Google, etc.)

**Naturaleza Federada y Autenticación de Empleados Externos:**

Debido a su naturaleza federada, Keycloak está diseñado para delegar la autenticación de usuarios internos (empleados) a sistemas de identidad externos. **Se espera que el backend de autenticación de empleados provenga externamente.** Por ejemplo, si la institución utiliza Azure Active Directory (Azure AD), Keycloak debe integrarse con Azure AD para que Keycloak delegue la autenticación a Azure AD. Este es un detalle de despliegue que debe abordarse en cada caso según las necesidades de la institución y los sistemas de identidad existentes.

**Configurabilidad:**

Keycloak es altamente configurable y la configuración descrita a continuación es una sugerencia que puede adaptarse a las necesidades de cada despliegue. Los reinos, clientes, flujos de autenticación y proveedores de identidad pueden configurarse según los requisitos específicos de cada cliente.

Como sugerencia, se configuran tres reinos:

- **Realm Interno:** Para usuarios internos y servicios de la aplicación
- **Realm de Clientes:** Para los clientes de la aplicación
- **Realm Data-Dagster:** Para acceso a herramientas de datos (Dagster)

De manera similar, se sugieren tres clientes de aplicación:

- **internal-service-account:** Para servicios internos de la aplicación
- **customer-service-account:** Para el portal de clientes
- **oauth2-proxy:** Para autenticación de OAuth2 Proxy

El flujo de autenticación para usuarios internos funciona de la siguiente manera: cuando un usuario accede al Panel de Administración (`admin.{domain}`), la aplicación redirige a Keycloak para la autenticación. Keycloak puede delegar la autenticación a un proveedor de identidad externo (p. ej., Azure AD, LDAP, etc.) o validar las credenciales directamente. Tras una autenticación exitosa, Keycloak genera tokens JWT que se utilizan para autenticar solicitudes a la API de GraphQL. Finalmente, Oathkeeper valida los tokens JWT antes de permitir el acceso a los recursos.

Para los clientes, el flujo de autenticación es similar: cuando un cliente accede al Portal de Clientes (`app.{domain}`), la aplicación redirige a Keycloak (Realm de Clientes) para la autenticación. Keycloak valida las credenciales y genera tokens JWT, que se utilizan para autenticar solicitudes a la API pública. Oathkeeper valida los tokens JWT antes de permitir el acceso a los recursos.

Los flujos de autenticación descritos son ejemplos y pueden variar según la configuración específica de cada despliegue, especialmente en lo que respecta a la integración con proveedores de identidad externos para usuarios internos.

#### 5.1.2 Oathkeeper

Oathkeeper actúa como un proxy de autenticación y autorización, proporcionando:

- Validación de tokens JWT
- Enrutamiento de solicitudes autenticadas
- Mutación de tokens (transformación de claims)
- Reglas de acceso basadas en URL y método HTTP
- Alta disponibilidad (2 réplicas por defecto)

Se configuran varias reglas de acceso:

- **admin-api:** Protege el endpoint GraphQL del Panel de Administración, requiere autenticación JWT
- **admin-ui:** Protege la interfaz del Panel de Administración, permite acceso sin autenticación (la autenticación es manejada por la aplicación)
- **customer-ui:** Protege el Portal del Cliente, permite acceso sin autenticación (la autenticación es manejada por la aplicación)
- **customer-api:** Protege la API pública del Portal del Cliente, requiere autenticación JWT

El flujo de validación funciona de la siguiente manera: cuando un cliente envía una solicitud con un token JWT en el encabezado Authorization, Oathkeeper extrae y valida el token JWT contra el JWKS de Keycloak. Oathkeeper verifica que el token no haya expirado y que el emisor sea válido, luego aplica las reglas de autorización según la URL y el método. Si la autorización es exitosa, Oathkeeper muta el token (opcional) y reenvía la solicitud al servicio upstream.

#### 5.1.3 OAuth2 Proxy

OAuth2 Proxy proporciona autenticación OAuth2/OIDC para aplicaciones que no soportan autenticación nativa. Se utiliza principalmente para proteger el acceso a Dagster.

El flujo de autenticación con OAuth2 Proxy funciona así: cuando un usuario accede a Dagster (`dagster.{domain}`), OAuth2 Proxy intercepta la solicitud y verifica si existe una sesión válida. Si no hay sesión, OAuth2 Proxy redirige a Keycloak para la autenticación. El usuario se autentica en Keycloak (puede usar Google como proveedor de identidad), y Keycloak redirige de vuelta a OAuth2 Proxy con un código de autorización. OAuth2 Proxy intercambia el código por tokens y crea una sesión, finalmente permitiendo el acceso a Dagster con encabezados de autenticación.
