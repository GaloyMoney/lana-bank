---
id: credit-ui
title: Interfaz de Gestión de Crédito
sidebar_position: 5
---

# Interfaz de Gestión de Facilidades de Crédito

Este documento describe los componentes y flujos de la interfaz de usuario para la gestión de facilidades de crédito.

## Visión General

La interfaz de crédito permite:

- Crear y gestionar facilidades de crédito
- Procesar desembolsos
- Registrar pagos
- Visualizar estado de la cartera

## Arquitectura de Componentes

```
┌─────────────────────────────────────────────────────────────────┐
│                    MÓDULO DE CRÉDITO                            │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    CreditModule                           │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Facilities │  │ Disbursals │  │  Payments  │         │  │
│  │  │   List     │  │   List     │  │    List    │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Facility   │  │ Disbursal  │  │  Payment   │         │  │
│  │  │  Detail    │  │  Form      │  │   Form     │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Credit GraphQL API                     │  │
│  │    (Queries: facilities, disbursals, payments, etc.)      │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Lista de Facilidades

### Componente FacilitiesList

```typescript
// components/credit/facilities-list.tsx
interface FacilitiesListProps {
  filter?: FacilityFilter;
  onSelect?: (facility: CreditFacility) => void;
}

export function FacilitiesList({ filter, onSelect }: FacilitiesListProps) {
  const { data, loading, fetchMore } = useCreditFacilitiesQuery({
    variables: {
      first: 20,
      filter,
    },
  });

  const columns: ColumnDef<CreditFacility>[] = [
    {
      accessorKey: 'publicId',
      header: 'ID',
    },
    {
      accessorKey: 'customer.name',
      header: 'Cliente',
    },
    {
      accessorKey: 'amount',
      header: 'Monto',
      cell: ({ row }) => formatCurrency(row.original.amount),
    },
    {
      accessorKey: 'status',
      header: 'Estado',
      cell: ({ row }) => <FacilityStatusBadge status={row.original.status} />,
    },
    {
      accessorKey: 'interestRate',
      header: 'Tasa',
      cell: ({ row }) => `${row.original.interestRate}%`,
    },
    {
      accessorKey: 'createdAt',
      header: 'Fecha Creación',
      cell: ({ row }) => formatDate(row.original.createdAt),
    },
  ];

  return (
    <DataTable
      columns={columns}
      data={data?.creditFacilities?.edges?.map(e => e.node) ?? []}
      onRowClick={onSelect}
    />
  );
}
```

### Filtros de Facilidades

```typescript
// components/credit/facility-filters.tsx
interface FacilityFiltersProps {
  value: FacilityFilter;
  onChange: (filter: FacilityFilter) => void;
}

export function FacilityFilters({ value, onChange }: FacilityFiltersProps) {
  return (
    <div className="flex gap-4 mb-4">
      <Select
        value={value.status}
        onValueChange={(status) => onChange({ ...value, status })}
      >
        <SelectTrigger className="w-[180px]">
          <SelectValue placeholder="Estado" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">Todos</SelectItem>
          <SelectItem value="ACTIVE">Activas</SelectItem>
          <SelectItem value="PENDING">Pendientes</SelectItem>
          <SelectItem value="CLOSED">Cerradas</SelectItem>
        </SelectContent>
      </Select>

      <Input
        placeholder="Buscar por cliente..."
        value={value.search}
        onChange={(e) => onChange({ ...value, search: e.target.value })}
        className="w-[250px]"
      />

      <DateRangePicker
        value={value.dateRange}
        onChange={(dateRange) => onChange({ ...value, dateRange })}
      />
    </div>
  );
}
```

## Detalle de Facilidad

### Componente FacilityDetail

```typescript
// components/credit/facility-detail.tsx
interface FacilityDetailProps {
  facilityId: string;
}

export function FacilityDetail({ facilityId }: FacilityDetailProps) {
  const { data, loading } = useCreditFacilityQuery({
    variables: { id: facilityId },
  });

  if (loading) return <FacilityDetailSkeleton />;

  const facility = data?.creditFacility;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <h1 className="text-2xl font-bold">{facility.publicId}</h1>
          <p className="text-muted-foreground">{facility.customer.name}</p>
        </div>
        <FacilityStatusBadge status={facility.status} size="lg" />
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          label="Monto Aprobado"
          value={formatCurrency(facility.amount)}
        />
        <StatCard
          label="Saldo Pendiente"
          value={formatCurrency(facility.outstanding)}
        />
        <StatCard
          label="Tasa de Interés"
          value={`${facility.interestRate}%`}
        />
        <StatCard
          label="Próximo Pago"
          value={formatDate(facility.nextPaymentDate)}
        />
      </div>

      {/* Tabs */}
      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Resumen</TabsTrigger>
          <TabsTrigger value="disbursals">Desembolsos</TabsTrigger>
          <TabsTrigger value="payments">Pagos</TabsTrigger>
          <TabsTrigger value="schedule">Calendario</TabsTrigger>
          <TabsTrigger value="collateral">Colateral</TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <FacilityOverview facility={facility} />
        </TabsContent>
        <TabsContent value="disbursals">
          <DisbursalsList facilityId={facilityId} />
        </TabsContent>
        <TabsContent value="payments">
          <PaymentsList facilityId={facilityId} />
        </TabsContent>
        <TabsContent value="schedule">
          <PaymentSchedule facilityId={facilityId} />
        </TabsContent>
        <TabsContent value="collateral">
          <CollateralInfo facilityId={facilityId} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
```

## Formulario de Desembolso

### Componente DisbursalForm

```typescript
// components/credit/disbursal-form.tsx
const disbursalSchema = z.object({
  amount: z.number().positive('El monto debe ser positivo'),
  reference: z.string().optional(),
});

interface DisbursalFormProps {
  facilityId: string;
  maxAmount: number;
  onSuccess?: () => void;
}

export function DisbursalForm({ facilityId, maxAmount, onSuccess }: DisbursalFormProps) {
  const [initiateDisbursal] = useInitiateDisbursalMutation();
  const form = useForm<z.infer<typeof disbursalSchema>>({
    resolver: zodResolver(disbursalSchema),
  });

  const onSubmit = async (data: z.infer<typeof disbursalSchema>) => {
    try {
      await initiateDisbursal({
        variables: {
          input: {
            creditFacilityId: facilityId,
            amount: data.amount * 100, // Convertir a centavos
            reference: data.reference,
          },
        },
      });
      toast.success('Desembolso iniciado correctamente');
      onSuccess?.();
    } catch (error) {
      toast.error('Error al iniciar el desembolso');
    }
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="amount"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Monto</FormLabel>
              <FormControl>
                <CurrencyInput
                  {...field}
                  max={maxAmount}
                  placeholder="0.00"
                />
              </FormControl>
              <FormDescription>
                Máximo disponible: {formatCurrency(maxAmount)}
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="reference"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Referencia (opcional)</FormLabel>
              <FormControl>
                <Input {...field} placeholder="Referencia externa" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline">
            Cancelar
          </Button>
          <Button type="submit">
            Iniciar Desembolso
          </Button>
        </div>
      </form>
    </Form>
  );
}
```
