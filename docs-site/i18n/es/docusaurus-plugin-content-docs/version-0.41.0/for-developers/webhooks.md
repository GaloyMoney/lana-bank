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

Consulta la [Referencia de Eventos](../apis/events/) para el catálogo completo de eventos de dominio.

## Seguridad

- Los payloads de webhook están firmados
- Verifica las firmas antes de procesar
- Usa solo endpoints HTTPS

## Política de Reintentos

Si tu endpoint no responde con un código de estado 2xx:

1. Lana reintentará la entrega del webhook
2. Los reintentos usan retroceso exponencial
3. Se garantiza que los eventos se entregarán al menos una vez

## Mejores Prácticas

- **Idempotencia**: Diseña los handlers para procesar eventos duplicados de forma segura
- **Confirmación rápida**: Devuelve 200 inmediatamente, procesa de forma asíncrona
- **Verificación de firma**: Siempre verifica las firmas de webhook antes de procesar
- **Registro**: Registra todos los eventos recibidos para depuración y auditoría
