---
id: index
title: Developer Guide
sidebar_position: 1
---

# Developer Guide

Welcome to the Lana developer documentation. This section covers everything you need to integrate with Lana's APIs.

## APIs Overview

Lana provides two GraphQL APIs:

| API | Purpose | Audience |
|-----|---------|----------|
| **[Admin API](admin-api/)** | Full system management - customers, credit, accounting, configuration | Internal systems, back-office applications |
| **[Customer API](customer-api/)** | Customer-facing operations - account info, facility status | Customer portals, mobile apps |

## Key Concepts

### GraphQL

Both APIs use GraphQL, providing:
- Strongly typed schemas
- Flexible queries - request exactly the data you need
- Real-time subscriptions for live updates

### Authentication

All API requests require authentication. See [Authentication](authentication) for setup details.

### Events

Lana uses event sourcing. You can subscribe to [Domain Events](events/) for real-time notifications of system changes.

## Quick Links

- [Admin API Reference](admin-api/) - Full admin operations and types
- [Customer API Reference](customer-api/) - Customer-facing operations
- [Domain Events](events/) - Event catalog and webhook integration
