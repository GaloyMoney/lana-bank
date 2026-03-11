---
sidebar_position: 3
title: APIs
description: Documentación de referencia de API para Lana Bank
---

# Referencia de API

Lana Bank expone dos APIs GraphQL y publica eventos de dominio para integración.

## APIs GraphQL

- **[API de Administración](admin-api)** — API completa para operaciones de back-office que incluye gestión de clientes, líneas de crédito, depósitos, contabilidad y configuración del sistema.

- **[API de Cliente](customer-api)** — API orientada al cliente para acceso a cuentas, solicitudes de préstamos, operaciones de depósito y gestión de documentos.

## Eventos de Dominio

- **[Eventos de Dominio](events/events.md)** — Eventos de dominio públicos publicados mediante el patrón transactional outbox, disponibles para integración, análisis y fines de auditoría.
