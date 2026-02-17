---
id: customer-portal
title: Portal del Cliente
sidebar_position: 3
---

# Portal del Cliente

Este documento describe la arquitectura y desarrollo del Portal del Cliente de Lana.

## Propósito

El Portal del Cliente permite a los clientes del banco:

- Ver el estado de sus cuentas
- Solicitar líneas de crédito
- Consultar saldos y movimientos
- Realizar retiros
- Descargar documentos y estados de cuenta

## Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                    PORTAL DEL CLIENTE                           │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Next.js App Router                     │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │  Home      │  │  Account   │  │  Credit    │         │  │
│  │  │  Page      │  │  Overview  │  │  Request   │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Transactions│ │  Documents │  │  Profile   │         │  │
│  │  │  Page      │  │  Page      │  │  Page      │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Apollo Client                          │  │
│  │                (Customer GraphQL API)                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                Keycloak (Customer Realm)                  │  │
│  │                    (Autenticación)                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Estructura del Proyecto

```
apps/customer-portal/
├── app/
│   ├── layout.tsx           # Layout principal
│   ├── page.tsx             # Página de inicio
│   ├── account/             # Resumen de cuenta
│   │   └── page.tsx
│   ├── transactions/        # Historial de transacciones
│   │   └── page.tsx
│   ├── credit/              # Solicitudes de crédito
│   │   ├── page.tsx         # Lista de facilidades
│   │   ├── request/         # Nueva solicitud
│   │   └── [id]/            # Detalle de facilidad
│   ├── documents/           # Documentos
│   │   └── page.tsx
│   └── profile/             # Perfil del cliente
│       └── page.tsx
├── components/
│   ├── layout/              # Componentes de layout
│   ├── account/             # Componentes de cuenta
│   ├── credit/              # Componentes de crédito
│   └── shared/              # Componentes compartidos
├── lib/
│   ├── apollo.ts            # Configuración Apollo
│   ├── keycloak.ts          # Configuración Keycloak
│   └── utils.ts             # Utilidades
└── generated/
    └── graphql.ts           # Tipos generados
```

## Páginas Principales

### Resumen de Cuenta

Muestra el estado general de la cuenta del cliente:

```typescript
// app/account/page.tsx
export default function AccountPage() {
  const { data } = useAccountOverviewQuery();

  return (
    <div className="space-y-6">
      <AccountBalanceCard balance={data?.account?.balance} />
      <RecentTransactions transactions={data?.account?.recentTransactions} />
      <CreditSummary facilities={data?.creditFacilities} />
    </div>
  );
}
```

### Solicitud de Crédito

Flujo para solicitar una nueva línea de crédito:

```typescript
// app/credit/request/page.tsx
export default function CreditRequestPage() {
  const [step, setStep] = useState(1);
  const [createProposal] = useCreateCreditProposalMutation();

  const handleSubmit = async (data: CreditRequestForm) => {
    await createProposal({
      variables: {
        input: {
          amount: data.amount,
          term: data.term,
          collateralType: data.collateralType,
        },
      },
    });
  };

  return (
    <CreditRequestWizard
      step={step}
      onStepChange={setStep}
      onSubmit={handleSubmit}
    />
  );
}
```

### Historial de Transacciones

Lista de movimientos de la cuenta:

```typescript
// app/transactions/page.tsx
export default function TransactionsPage() {
  const { data, fetchMore } = useTransactionsQuery({
    variables: { first: 20 },
  });

  return (
    <div>
      <TransactionFilters />
      <TransactionList transactions={data?.transactions} />
      <LoadMoreButton
        onClick={() => fetchMore({
          variables: {
            after: data?.transactions?.pageInfo?.endCursor,
          },
        })}
      />
    </div>
  );
}
```

## Componentes del Portal

### Balance Card

```typescript
// components/account/balance-card.tsx
interface BalanceCardProps {
  balance: {
    available: number;
    pending: number;
    total: number;
  };
}

export function BalanceCard({ balance }: BalanceCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Saldo de Cuenta</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-4">
          <Stat label="Disponible" value={formatCurrency(balance.available)} />
          <Stat label="Pendiente" value={formatCurrency(balance.pending)} />
          <Stat label="Total" value={formatCurrency(balance.total)} />
        </div>
      </CardContent>
    </Card>
  );
}
```

### Credit Facility Card

```typescript
// components/credit/facility-card.tsx
export function FacilityCard({ facility }: FacilityCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Línea de Crédito</CardTitle>
        <Badge variant={getStatusVariant(facility.status)}>
          {facility.status}
        </Badge>
      </CardHeader>
      <CardContent>
        <dl className="space-y-2">
          <dt>Monto Aprobado</dt>
          <dd>{formatCurrency(facility.amount)}</dd>
          <dt>Saldo Disponible</dt>
          <dd>{formatCurrency(facility.available)}</dd>
          <dt>Tasa de Interés</dt>
          <dd>{facility.interestRate}%</dd>
        </dl>
      </CardContent>
      <CardFooter>
        <Button asChild>
          <Link href={`/credit/${facility.id}`}>Ver Detalles</Link>
        </Button>
      </CardFooter>
    </Card>
  );
}
```

## Flujos de Usuario

### Solicitud de Desembolso

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Ver línea   │───▶│  Solicitar   │───▶│  Pendiente   │
│  de crédito  │    │  desembolso  │    │  aprobación  │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                    ┌──────────────┐    ┌──────────────┐
                    │  Desembolso  │◀───│   Aprobado   │
                    │  recibido    │    │              │
                    └──────────────┘    └──────────────┘
```

### Solicitud de Retiro

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Ver saldo   │───▶│  Solicitar   │───▶│  Pendiente   │
│  disponible  │    │  retiro      │    │  aprobación  │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                    ┌──────────────┐    ┌──────────────┐
                    │   Fondos     │◀───│   Aprobado   │
                    │   enviados   │    │              │
                    └──────────────┘    └──────────────┘
```

## Autenticación

### Configuración de Keycloak

```typescript
// lib/keycloak.ts
import Keycloak from 'keycloak-js';

export const keycloak = new Keycloak({
  url: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
  realm: 'customer',
  clientId: 'customer-portal',
});
```

### Proceso de Login

1. Usuario accede al portal
2. Redirigido a Keycloak para autenticación
3. Ingresa credenciales (email/contraseña)
4. Keycloak valida y genera JWT
5. Usuario redirigido al portal con sesión activa

## Responsividad

El portal está diseñado para funcionar en múltiples dispositivos:

```typescript
// Breakpoints de Tailwind
// sm: 640px - Móviles grandes
// md: 768px - Tablets
// lg: 1024px - Laptops
// xl: 1280px - Desktops

export function AccountLayout({ children }) {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <aside className="hidden md:block">
          <AccountSidebar />
        </aside>
        <main className="md:col-span-2">
          {children}
        </main>
      </div>
    </div>
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
```

### Variables de Entorno

```env
# .env.local
NEXT_PUBLIC_GRAPHQL_URL=http://app.localhost:4455/graphql
NEXT_PUBLIC_KEYCLOAK_URL=http://localhost:8081
NEXT_PUBLIC_KEYCLOAK_REALM=customer
NEXT_PUBLIC_KEYCLOAK_CLIENT_ID=customer-portal
```

