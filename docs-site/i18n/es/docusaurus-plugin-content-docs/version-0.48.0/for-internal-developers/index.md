---
id: index
title: Guía Interna para Desarrolladores
sidebar_position: 1
---

# Guía Interna para Desarrolladores

Bienvenido a la documentación interna para desarrolladores de Lana. Esta sección cubre todo lo que necesitas para trabajar en el código base de lana-bank: configuración local, arquitectura interna, desarrollo frontend y patrones de servicios de dominio.

## Primeros Pasos

¿Nuevo en el código base? Comienza aquí:

- [Configuración de Desarrollo Local](local-development) — Configura un entorno de desarrollo funcional en minutos
- [Autenticación (Local)](authentication-local) — Reinos de Keycloak, flujos de inicio de sesión, credenciales de prueba
- [Autorización](authorization) — Modelo RBAC de Casbin, roles y permisos

## Desarrollo Frontend

Construye y amplía el panel de administración y el portal del cliente:

- [Aplicaciones Frontend](frontend/) — Stack tecnológico, patrones y estructura del proyecto
- [Panel de Administración](frontend/admin-panel) — Arquitectura y desarrollo del panel de administración
- [Portal del Cliente](frontend/customer-portal) — Arquitectura del portal del cliente
- [Componentes Compartidos](frontend/shared-components) — Biblioteca de componentes de interfaz
- [Interfaz de Crédito](frontend/credit-ui) — Interfaz de gestión de líneas de crédito
- [Desarrollo GraphQL](graphql-development) — Configuración de Apollo Client, generación de código y endpoints locales

## Arquitectura de Dominio

Comprende el diseño interno de cada módulo:

- [Servicios de Dominio](domain-services) — Estructura de módulos DDD e interacciones
- [Sistema de Eventos](event-system) — Event sourcing, patrón outbox, eventos públicos vs privados
- [Trabajos en Segundo Plano](background-jobs) — Procesamiento de trabajos, programación y trabajos específicos
- [Integración con Cala Ledger](cala-ledger-integration) — Motor de contabilidad de doble entrada
- [Custodia y Portafolio](custody-portfolio) — Integración con BitGo/Komainu, gestión de colateral
- [Prueba de Auto-Custodia en Signet](self-custody-signet) — Monederos Signet locales, configuración de xpub y financiamiento de líneas pendientes

## Infraestructura y Operaciones

- [Servicios de Infraestructura](infrastructure-services) — Dependencias externas y capas de servicio
- [Observabilidad](observability) — OpenTelemetry, rastreo, Honeycomb
- [Sistema de Auditoría](audit-system) — Registro de autorizaciones y cumplimiento normativo
- [Configuración](configuration) — Sistema de configuración de dominio y macros
