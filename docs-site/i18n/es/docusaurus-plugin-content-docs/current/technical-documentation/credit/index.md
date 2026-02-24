---
id: index
title: Módulo de Crédito
sidebar_position: 1
---

# Ciclo de Vida del Módulo de Crédito

```mermaid
graph TD
    subgraph Clients["Aplicaciones Cliente"]
        ADMIN["admin-panel<br/>(Next.js)"]
        CUST["customer-portal<br/>(Next.js)"]
    end

    subgraph Backend["Servicios Backend"]
        LANACLI["lana-cli<br/>Punto de Entrada"]
        CS_API["customer-server<br/>GraphQL API"]
        AS_API["admin-server<br/>GraphQL API"]
    end

    subgraph GW["Capa API Gateway"]
        OAT["Oathkeeper<br/>Gateway JWT"]
    end

    subgraph Core["Dominios de Negocio"]
        LA["lana-app<br/>Orquestador"]
        CCUS["core-custody"]
        CDEP["core-deposit"]
        CCRED["core-credit"]
        CCUST["core-customer"]
        CACCT["core-accounting"]
        GOV["governance"]
    end

    subgraph Ledger
        CALA["cala-ledger"]
        PG[("PostgreSQL")]
    end

    subgraph External["Integraciones Externas"]
        BITGO["BitGo/Komainu<br/>Custodia"]
        SUMSUB["Sumsub<br/>KYC/AML"]
    end

    ADMIN --> OAT
    CUST --> OAT
    OAT --> AS_API
    OAT --> CS_API
    LANACLI --> LA
    AS_API --> LA
    CS_API --> LA
    LA --> CCUS
    LA --> CDEP
    LA --> CCRED
    LA --> CCUST
    LA --> CACCT
    LA --> GOV
    CACCT --> CALA
    CALA --> PG
    CCUS --> BITGO
    CCUST --> SUMSUB
```

```mermaid
flowchart LR
    %% Préstamo = Facilidad de Crédito (n1) + Desembolso (S_D)
    subgraph Prestamo["Préstamo"]
    direction LR
        n1["Facilidad de Crédito <br/>&lt;CicloDeDevengoDeIntereses&gt;"]

        subgraph S_D["Desembolso"]
        direction LR
            d1["Desembolso 1"]:::small
            d2["Desembolso 2"]:::small
        end
    end

    subgraph S_O["Obligación"]
    direction LR
        o1["Obligación 1"]:::small
        o2["Obligación 2"]:::small
        o3["Obligación 3"]:::small
    end

    subgraph S_R["."]
    direction LR
        subgraph S_R1["Pago 1"]
        direction LR
            r1["AsignaciónDePago 1"]:::small
            r2["AsignaciónDePago 2"]:::small
        end
        subgraph S_R2["Pago 2"]
        direction LR
            r3["AsignaciónDePago 3"]:::small
        end
        r3["AsignaciónDePago 3"]:::small
    end

    n1 --> S_D --> S_O

    o1 --> r1
    o2 --> r2
    o2 --> r3
    o3 --> r3

    classDef small stroke:#999,stroke-width:1px;
    style Prestamo stroke:#666,stroke-width:2px,stroke-dasharray:6 3,fill:none;
```

> Una [`FacilidadDeCrédito`](./facility) adelanta fondos a un prestatario a través de uno o más [`Desembolsos`](./disbursal).
  Cada desembolso crea las correspondientes [`Obligaciones`](./obligation) (para *Principal* o cualquier *Interés Devengado*) que el prestatario debe pagar.
  Cuando el prestatario hace un [`Pago`](./payment), se asigna a obligaciones específicas a través de registros de [`AsignaciónDePago`](./payment#asignación-de-pago).
  Los [`Términos`](./terms) definen las tasas de interés, cronogramas y otras reglas que rigen la facilidad y sus obligaciones.
  Una vez que cada obligación está completamente satisfecha, la facilidad de crédito se cierra automáticamente.

## Páginas del Módulo

| Página | Descripción |
|------|-------------|
| [Facilidades de Crédito](./facility) | Creación de propuestas, flujo de aprobación, colateralización, activación y estados de la facilidad |
| [Desembolsos](./disbursal) | Disposición de fondos desde facilidades activas, flujo de aprobación y liquidación |
| [Obligaciones](./obligation) | Seguimiento de deuda, tipos de obligación, estados de ciclo de vida y parámetros de tiempo |
| [Pagos](./payment) | Procesamiento de pagos, reglas de prioridad de asignación e impacto contable |
| [Términos](./terms) | Tasas de interés, cronogramas de comisiones, intervalos de tiempo, umbrales CVL y plantillas de términos |
| [Procesamiento de Intereses](./interest-process) | Devengo diario, ciclos mensuales, creación de obligaciones y asientos contables |
| [Libro Mayor](./ledger.md) | Resumen de conjuntos de cuentas y plantillas de transacción |
