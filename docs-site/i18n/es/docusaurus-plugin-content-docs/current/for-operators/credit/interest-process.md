---
id: interest-process
title: Proceso de Intereses
sidebar_position: 7
---

# Proceso de Intereses

El interés en una facilidad de crédito se devenga periódicamente y se captura como nuevas obligaciones.
El proceso es orquestado por un par de trabajos:

1. **Trabajo de Devengo de Intereses (`interest-accrual`)** – confirma el interés devengado para el período actual.
   Registra una entrada en el libro mayor para el monto devengado y se reprograma para el siguiente
   período de devengo. Después del período final en un ciclo, genera el trabajo del ciclo.

2. **Trabajo del Ciclo de Devengo de Intereses (`interest-accrual-cycle`)** – registra los devengos
   confirmados para el ciclo completado. Este trabajo crea una `Obligación` de tipo
   *Interés* y programa el primer devengo del siguiente ciclo.

Cada obligación de interés está vinculada de vuelta a la facilidad para que cuando un prestatario hace un
`Pago`, los registros de `AsignaciónDePago` puedan reducir el saldo de intereses pendiente.
