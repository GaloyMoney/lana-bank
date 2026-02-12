---
id: index
title: Internal Developer Guide
sidebar_position: 1
---

# Internal Developer Guide

Welcome to the Lana internal developer documentation. This section covers everything you need to work on the lana-bank codebase — local setup, architecture internals, frontend development, and domain service patterns.

## Getting Started

New to the codebase? Start here:

- [Local Development Setup](local-development) — Get a working dev environment in minutes
- [Authentication (Local)](authentication-local) — Keycloak realms, login flows, test credentials
- [Authorization](authorization) — Casbin RBAC model, roles, and permissions

## Frontend Development

Build and extend the admin panel and customer portal:

- [Frontend Applications](frontend/) — Tech stack, patterns, and project structure
- [Admin Panel](frontend/admin-panel) — Admin panel architecture and development
- [Customer Portal](frontend/customer-portal) — Customer portal architecture
- [Shared Components](frontend/shared-components) — UI component library
- [Credit UI](frontend/credit-ui) — Credit facility management interface
- [GraphQL Development](graphql-development) — Apollo Client setup, codegen, and local endpoints

## Domain Architecture

Understand the internal design of each module:

- [Domain Services](domain-services) — DDD module structure and interactions
- [Event System](event-system) — Event sourcing, outbox pattern, public vs private events
- [Background Jobs](background-jobs) — Job processing, scheduling, and specific jobs
- [Cala Ledger Integration](cala-ledger-integration) — Double-entry accounting engine
- [Custody & Portfolio](custody-portfolio) — BitGo/Komainu integration, collateral management

## Infrastructure & Operations

- [Infrastructure Services](infrastructure-services) — External dependencies and service layers
- [Observability](observability) — OpenTelemetry, tracing, Honeycomb
- [Audit System](audit-system) — Authorization logging and compliance
- [Configuration](configuration) — Domain config system and macros
