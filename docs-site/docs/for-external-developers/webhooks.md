---
id: webhooks
title: Webhooks
sidebar_position: 5
---

# Webhooks

Receive real-time notifications when events occur in Lana.

## Overview

Lana can notify your systems when important events happen:

- Customer onboarding status changes
- Credit facility state transitions
- Payment processing events
- Approval workflow updates

## How It Works

1. Register a webhook endpoint URL
2. Subscribe to event types you care about
3. Lana sends HTTP POST requests when events occur
4. Your system processes and acknowledges events

## Event Format

Events are delivered as JSON:

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

## Available Events

See the [Events Reference](../apis/events/events.md) for the complete catalog of domain events.

## Security

- Webhook payloads are signed
- Verify signatures before processing
- Use HTTPS endpoints only

## Retry Policy

If your endpoint fails to respond with a 2xx status code:

1. Lana will retry the webhook delivery
2. Retries use exponential backoff
3. Events are guaranteed to be delivered at least once

## Best Practices

- **Idempotency**: Design handlers to safely process duplicate events
- **Quick acknowledgment**: Return 200 immediately, process asynchronously
- **Signature verification**: Always verify webhook signatures before processing
- **Logging**: Log all received events for debugging and audit trails
