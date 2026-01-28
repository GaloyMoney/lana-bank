---
id: webhooks
title: Webhooks
sidebar_position: 4
---

# Webhooks

Recibe notificaciones en tiempo real cuando ocurren eventos en Lana.

## Resumen

Lana puede notificar a tus sistemas cuando ocurren eventos importantes:

- Cambios en el estado de incorporación de clientes
- Transiciones de estado de facilidades de crédito
- Eventos de procesamiento de pagos
- Actualizaciones de flujos de trabajo de aprobación

## Cómo Funciona

1. Registra una URL de endpoint de webhook
2. Suscríbete a los tipos de eventos que te interesan
3. Lana envía solicitudes HTTP POST cuando ocurren los eventos
4. Tu sistema procesa y confirma la recepción de los eventos

## Formato de Eventos

Los eventos se entregan como JSON:

```json
{
  "event_type": "credit_facility.activated",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "facility_id": "...",
    "customer_id": "...",
    "amount": "..."
  }
}
```

## Eventos Disponibles

Consulta la [Referencia de Eventos](events/) para el catálogo completo de eventos de dominio.

## Seguridad

- Los payloads de webhook están firmados
- Verifica las firmas antes de procesar
- Usa solo endpoints HTTPS

*[Documentación detallada de webhooks próximamente - se añadirá del manual técnico]*
