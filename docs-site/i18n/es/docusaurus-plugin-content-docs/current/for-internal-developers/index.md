---
id: index
title: Guía interna para desarrolladores
sidebar_position: 1
---

# Guía interna para desarrolladores

Bienvenido a la documentación interna para desarrolladores de Lana. Esta sección cubre todo lo que necesitas para trabajar en el código base de lana-bank: configuración local, arquitectura interna, desarrollo frontend y patrones de servicios de dominio.

## Primeros pasos

¿Nuevo en el código base? Empieza aquí:

- [Configuración de desarrollo local](local-development): configura un entorno de desarrollo funcional en minutos
- [Autenticación (local)](authentication-local): reinos de Keycloak, flujos de inicio de sesión, credenciales de prueba
- [Autorización](authorization): modelo RBAC de Casbin, roles y permisos

## Desarrollo frontend

Construye y amplía el panel de administración y el portal del cliente:

- [Aplicaciones frontend](frontend/): stack tecnológico, patrones y estructura del proyecto
- [Panel de administración](frontend/admin-panel): arquitectura y desarrollo del panel de administración
- [Portal del cliente](frontend/customer-portal): arquitectura del portal del cliente
- [Componentes compartidos](frontend/shared-components): biblioteca de componentes de UI
- [Credit UI](frontend/credit-ui): interfaz de gestión de líneas de crédito
- [Desarrollo GraphQL](graphql-development): configuración de Apollo Client, generación de código y endpoints locales

## Arquitectura de dominio

Comprende el diseño interno de cada módulo:

- [Servicios de Dominio](domain-services) — Estructura del módulo DDD e interacciones
- [Sistema de Eventos](event-system) — Event sourcing, patrón outbox, eventos públicos vs privados
- [Trabajos en Segundo Plano](background-jobs) — Procesamiento de trabajos, programación y trabajos específicos
- [Integración con Cala Ledger](cala-ledger-integration) — Motor de contabilidad de partida doble
- [Custodia y Portafolio](custody-portfolio) — Integración con BitGo/Komainu/Bitfinex, gestión de colaterales

## Infraestructura y operaciones

- [Servicios de infraestructura](infrastructure-services): dependencias externas y capas de servicio
- [Observabilidad](observability): OpenTelemetry, trazabilidad, Honeycomb
- [Sistema de auditoría](audit-system): registro de autorizaciones y cumplimiento normativo
- [Configuración](configuration): sistema de configuración de dominio y macros
