---
id: closing
title: Cierre
sidebar_position: 2
---

# Cierre - Transferir Ingreso Neto del EstadoDeResultados al BalanceGeneral

## Compensación de Saldos Efectivos de Cuentas del EstadoDeResultados

### Tipos de Saldo Normal Negativo

Los saldos efectivos de Cala pueden ser negativos. Compensar saldos normales negativos con una entrada de transacción de cierre funciona diferente que un tipo de saldo normal positivo.

```rust
pub fn settled(&self) -> Decimal {
    if self.direction == DebitOrCredit::Credit {
        self.details.settled.cr_balance - self.details.settled.dr_balance
    } else {
        self.details.settled.dr_balance - self.details.settled.cr_balance
    }
}
```

### Cuentas Contra

Una cuenta contra es una `Cuenta` de un `ConjuntoDeCuentas` en el `PlanDeCuentas`, que tiene un tipo de saldo normal diferente al de su padre.

Ejemplo, cuentas de operador de `lana-bank` con una provisión para pérdidas de préstamos. En un mes dado, las pérdidas de préstamos realizadas fueron menores que la provisión para un período. Un Contador/CFO, hará una transacción manual con una entrada acreditando una cuenta de `Gasto` de saldo normal crédito - reduciendo las pérdidas realizadas.
