---
id: payment
title: Pago
sidebar_position: 5
---

# Pago

Captura los fondos remitidos por el prestatario hacia una facilidad.
Cada pago se desglosa en una o más asignaciones que liquidan obligaciones específicas en orden de prioridad.
Los eventos emitidos desde los pagos actualizan los planes de pago, saldos y cierran obligaciones una vez que están completamente cubiertas.

## Asignación de Pago

Cuando se recibe un pago, se asigna a las obligaciones pendientes basándose en reglas de prioridad. Cada registro de `AsignaciónDePago` vincula una porción del pago a una obligación específica, reduciendo su saldo pendiente.
