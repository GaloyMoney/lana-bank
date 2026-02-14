---
id: index
title: Aplicaciones Frontend
sidebar_position: 1
---

# Aplicaciones Frontend

Este documento describe las aplicaciones frontend de Lana, su arquitectura y patrones de desarrollo.

## Visión General

Lana incluye dos aplicaciones frontend principales:

| Aplicación | Propósito | Usuarios |
|------------|-----------|----------|
| Panel de Administración | Gestión del banco | Personal administrativo |
| Portal del Cliente | Autoservicio | Clientes del banco |

## Stack Tecnológico

```
┌─────────────────────────────────────────────────────────────────┐
│                    STACK FRONTEND                               │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │    Next.js      │  │     React       │  │   TypeScript    │ │
│  │    (Framework)  │  │    (UI Library) │  │   (Lenguaje)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Apollo Client  │  │   Tailwind CSS  │  │  shadcn/ui      │ │
│  │  (GraphQL)      │  │   (Estilos)     │  │  (Componentes)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Keycloak JS   │  │   React Hook    │  │   Zod           │ │
│  │   (Auth)        │  │   Form          │  │   (Validación)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Arquitectura de Aplicaciones

```
┌─────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA FRONTEND                        │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                       Next.js App                         │  │
│  │  ┌────────────────┐  ┌────────────────┐                  │  │
│  │  │   Pages/       │  │   Components/  │                  │  │
│  │  │   Routes       │  │   UI           │                  │  │
│  │  └────────────────┘  └────────────────┘                  │  │
│  │  ┌────────────────┐  ┌────────────────┐                  │  │
│  │  │   Hooks/       │  │   Lib/         │                  │  │
│  │  │   Custom       │  │   Utilities    │                  │  │
│  │  └────────────────┘  └────────────────┘                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Apollo Client                          │  │
│  │              (Gestión de estado y caché)                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    GraphQL APIs                           │  │
│  │              (Admin Server / Customer Server)             │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Estructura de Directorios

```
apps/
├── admin-panel/           # Panel de Administración
│   ├── app/               # Next.js App Router
│   ├── components/        # Componentes React
│   ├── lib/               # Utilidades y configuración
│   ├── hooks/             # Custom hooks
│   └── generated/         # Código generado (GraphQL)
│
├── customer-portal/       # Portal del Cliente
│   ├── app/               # Next.js App Router
│   ├── components/        # Componentes React
│   ├── lib/               # Utilidades y configuración
│   ├── hooks/             # Custom hooks
│   └── generated/         # Código generado (GraphQL)
│
└── shared/                # Código compartido
    ├── ui/                # Componentes de UI
    └── utils/             # Utilidades comunes
```

## Patrones de Desarrollo

### Server Components vs Client Components

Next.js 13+ usa React Server Components por defecto:

```typescript
// Server Component (por defecto)
// app/customers/page.tsx
export default async function CustomersPage() {
  const customers = await fetchCustomers();
  return <CustomerList customers={customers} />;
}

// Client Component (interactivo)
// components/customer-form.tsx
'use client';

export function CustomerForm() {
  const [name, setName] = useState('');
  // ...
}
```

### Gestión de Estado

- **Apollo Client**: Estado del servidor (datos de GraphQL)
- **React Context**: Estado global de UI
- **useState/useReducer**: Estado local de componentes

### Autenticación

```typescript
// hooks/useAuth.ts
export function useAuth() {
  const { keycloak, initialized } = useKeycloak();

  return {
    isAuthenticated: keycloak.authenticated,
    token: keycloak.token,
    login: () => keycloak.login(),
    logout: () => keycloak.logout(),
  };
}
```

## Desarrollo Local

### Iniciar Aplicaciones

```bash
# Panel de Administración
cd apps/admin-panel
pnpm dev

# Portal del Cliente
cd apps/customer-portal
pnpm dev
```

### URLs de Desarrollo

| Aplicación | URL |
|------------|-----|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |

## Documentación Relacionada

- [Panel de Administración](admin-panel) - Documentación del panel admin
- [Portal del Cliente](customer-portal) - Documentación del portal de clientes
- [Componentes Compartidos](shared-components) - Biblioteca de UI
- [Interfaz de Crédito](credit-ui) - Gestión de facilidades de crédito

