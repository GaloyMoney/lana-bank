---
id: admin-panel
title: Panel de Administración
sidebar_position: 2
---

# Panel de Administración

Este documento describe la arquitectura y desarrollo del Panel de Administración de Lana.

## Propósito

El Panel de Administración es la interfaz principal para el personal del banco:

- Gestión de clientes
- Administración de líneas de crédito
- Operaciones de depósito y retiro
- Aprobaciones y gobernanza
- Reportes financieros

## Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                    PANEL DE ADMINISTRACIÓN                      │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Next.js App Router                     │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Dashboard  │  │ Customers  │  │  Credit    │         │  │
│  │  │ Page       │  │ Module     │  │  Module    │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Deposits   │  │ Approvals  │  │  Reports   │         │  │
│  │  │ Module     │  │ Module     │  │  Module    │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Apollo Client                          │  │
│  │                 (Admin GraphQL API)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Keycloak (Admin Realm)                 │  │
│  │                      (Autenticación)                      │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Estructura del Proyecto

```
apps/admin-panel/
├── app/
│   ├── layout.tsx           # Layout principal
│   ├── page.tsx             # Dashboard
│   ├── customers/           # Módulo de clientes
│   │   ├── page.tsx         # Lista de clientes
│   │   └── [id]/            # Detalle de cliente
│   ├── credit/              # Módulo de crédito
│   │   ├── facilities/      # Facilidades
│   │   ├── disbursals/      # Desembolsos
│   │   └── payments/        # Pagos
│   ├── deposits/            # Módulo de depósitos
│   ├── approvals/           # Módulo de aprobaciones
│   └── reports/             # Módulo de reportes
├── components/
│   ├── layout/              # Componentes de layout
│   ├── customers/           # Componentes de clientes
│   ├── credit/              # Componentes de crédito
│   └── shared/              # Componentes compartidos
├── lib/
│   ├── apollo.ts            # Configuración Apollo
│   ├── keycloak.ts          # Configuración Keycloak
│   └── utils.ts             # Utilidades
└── generated/
    └── graphql.ts           # Tipos generados
```

## Módulos Principales

### Dashboard

El dashboard proporciona una visión general del estado del sistema:

- Resumen de cartera
- Aprobaciones pendientes
- Alertas y notificaciones
- KPIs principales

### Módulo de Clientes

Gestión completa del ciclo de vida del cliente:

```typescript
// app/customers/page.tsx
import { CustomerList } from '@/components/customers/customer-list';
import { useCustomersQuery } from '@/generated/graphql';

export default function CustomersPage() {
  const { data, loading } = useCustomersQuery({
    variables: { first: 20 },
  });

  return <CustomerList customers={data?.customers} loading={loading} />;
}
```

### Módulo de Crédito

Administración de facilidades de crédito:

- Lista de facilidades
- Detalle de facilidad
- Proceso de desembolso
- Registro de pagos

### Módulo de Aprobaciones

Flujos de trabajo de aprobación:

```typescript
// components/approvals/approval-list.tsx
export function ApprovalList() {
  const { data } = usePendingApprovalsQuery();

  return (
    <div>
      {data?.pendingApprovals.map((approval) => (
        <ApprovalCard
          key={approval.id}
          approval={approval}
          onApprove={() => handleApprove(approval.id)}
          onReject={() => handleReject(approval.id)}
        />
      ))}
    </div>
  );
}
```

## Componentes Comunes

### Layout

```typescript
// components/layout/sidebar.tsx
export function Sidebar() {
  const navigation = [
    { name: 'Dashboard', href: '/', icon: HomeIcon },
    { name: 'Clientes', href: '/customers', icon: UsersIcon },
    { name: 'Crédito', href: '/credit', icon: CreditCardIcon },
    { name: 'Depósitos', href: '/deposits', icon: BankIcon },
    { name: 'Aprobaciones', href: '/approvals', icon: CheckIcon },
    { name: 'Reportes', href: '/reports', icon: ChartIcon },
  ];

  return (
    <nav className="sidebar">
      {navigation.map((item) => (
        <NavLink key={item.name} {...item} />
      ))}
    </nav>
  );
}
```

### Tablas de Datos

```typescript
// components/shared/data-table.tsx
interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  onRowClick?: (row: T) => void;
  pagination?: boolean;
}

export function DataTable<T>({ data, columns, ...props }: DataTableProps<T>) {
  return (
    <Table>
      <TableHeader columns={columns} />
      <TableBody data={data} columns={columns} {...props} />
    </Table>
  );
}
```

### Formularios

```typescript
// components/customers/customer-form.tsx
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

const schema = z.object({
  email: z.string().email(),
  customerType: z.enum(['INDIVIDUAL', 'COMPANY']),
});

export function CustomerForm({ onSubmit }: CustomerFormProps) {
  const form = useForm({
    resolver: zodResolver(schema),
  });

  return (
    <Form {...form}>
      <FormField name="email" label="Email" />
      <FormField name="customerType" label="Tipo de Cliente" />
      <Button type="submit">Crear Cliente</Button>
    </Form>
  );
}
```

## Autenticación y Autorización

### Configuración de Keycloak

```typescript
// lib/keycloak.ts
import Keycloak from 'keycloak-js';

export const keycloak = new Keycloak({
  url: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
  realm: 'admin',
  clientId: 'admin-panel',
});
```

### Protección de Rutas

```typescript
// components/auth/protected-route.tsx
export function ProtectedRoute({ children, requiredRole }) {
  const { isAuthenticated, hasRole } = useAuth();

  if (!isAuthenticated) {
    return <LoginRedirect />;
  }

  if (requiredRole && !hasRole(requiredRole)) {
    return <AccessDenied />;
  }

  return children;
}
```

## Internacionalización

El panel soporta múltiples idiomas usando `next-intl`:

```typescript
// app/[locale]/layout.tsx
import { NextIntlClientProvider } from 'next-intl';

export default async function LocaleLayout({ children, params: { locale } }) {
  const messages = await import(`@/messages/${locale}.json`);

  return (
    <NextIntlClientProvider locale={locale} messages={messages}>
      {children}
    </NextIntlClientProvider>
  );
}
```

## Desarrollo

### Comandos

```bash
# Desarrollo
pnpm dev

# Build de producción
pnpm build

# Lint
pnpm lint

# Tests
pnpm test

# Generar tipos GraphQL
pnpm codegen
```

### Variables de Entorno

```env
# .env.local
NEXT_PUBLIC_GRAPHQL_URL=http://admin.localhost:4455/graphql
NEXT_PUBLIC_KEYCLOAK_URL=http://localhost:8081
NEXT_PUBLIC_KEYCLOAK_REALM=admin
NEXT_PUBLIC_KEYCLOAK_CLIENT_ID=admin-panel
```

