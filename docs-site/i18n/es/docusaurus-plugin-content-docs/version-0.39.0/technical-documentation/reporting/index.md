---
id: index
title: Sistema de Reportes
sidebar_position: 1
---

# Sistema de Reportes Financieros

El sistema de reportes proporciona informes financieros para la gestión operativa y el cumplimiento regulatorio.

## Propósito

El sistema de reportes permite:
- Generación de estados financieros
- Informes regulatorios
- Análisis de cartera
- Reportes de auditoría

## Arquitectura de Reportes

```
┌─────────────────────────────────────────────────────────────────┐
│                    SISTEMA DE REPORTES                          │
│                                                                  │
│  ┌─────────────────┐                                            │
│  │   Cala Ledger   │                                            │
│  │  (Datos origen) │                                            │
│  └─────────────────┘                                            │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Data Pipeline                         │   │
│  │              (Meltano + dbt + Dagster)                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │    BigQuery     │  │   Dashboards    │  │    Reports      │ │
│  │  (Data Warehouse)│  │   (Análisis)    │  │  (Exportación)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Tipos de Reportes

### Reportes Financieros

| Reporte | Descripción | Frecuencia |
|---------|-------------|------------|
| Balanza de Comprobación | Saldos de todas las cuentas | Diario/Mensual |
| Balance General | Estado de posición financiera | Mensual |
| Estado de Resultados | Ingresos y gastos | Mensual |

### Reportes Operativos

| Reporte | Descripción | Frecuencia |
|---------|-------------|------------|
| Cartera de Crédito | Estado de líneas de crédito | Diario |
| Depósitos | Posición de depósitos | Diario |
| Colateral | Valoración de garantías | Diario |

### Reportes Regulatorios

| Reporte | Descripción | Frecuencia |
|---------|-------------|------------|
| Concentración de Crédito | Exposición por cliente | Mensual |
| Morosidad | Cartera vencida | Mensual |
| Capital | Ratios de capital | Trimestral |

## Flujo de Datos

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Eventos    │───▶│    ETL       │───▶│   Data       │
│  del Sistema │    │   (Meltano)  │    │  Warehouse   │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │ Transformación│
                                        │    (dbt)     │
                                        └──────────────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │   Reportes   │
                                        │  Generados   │
                                        └──────────────┘
```

## Acceso a Reportes

### Panel de Administración

1. Navegar a **Reportes**
2. Seleccionar tipo de reporte
3. Configurar parámetros:
   - Período
   - Filtros
   - Formato de salida
4. Generar reporte

### Formatos de Exportación

| Formato | Uso |
|---------|-----|
| PDF | Presentación formal |
| Excel | Análisis adicional |
| CSV | Integración con otros sistemas |

## Programación de Reportes

### Reportes Automáticos

Configurar generación automática de reportes:

1. Navegar a **Configuración** > **Reportes Programados**
2. Seleccionar reporte
3. Configurar:
   - Frecuencia (diaria, semanal, mensual)
   - Hora de ejecución
   - Destinatarios
4. Activar programación

## Documentación Relacionada

- [Informes Financieros](financial-reports) - Detalle de reportes financieros

