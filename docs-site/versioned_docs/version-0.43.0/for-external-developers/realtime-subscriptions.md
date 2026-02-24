---
id: realtime-subscriptions
title: Real-time Subscriptions
sidebar_position: 5
---

# Real-time Subscriptions

Lana provides real-time notifications through **GraphQL subscriptions** over WebSocket. Instead of polling the API for changes, your application can subscribe to specific events and receive updates the moment they occur.

## How It Works

GraphQL subscriptions use a persistent WebSocket connection to push events from the server to your client.

**Endpoint:** `ws://admin.localhost:4455/graphql` (development) or `wss://<your-domain>/graphql` (production)

**Protocol:** GraphQL over WebSocket (`graphql-transport-ws`)

**Authentication:** The WebSocket connection requires the same JWT authentication as regular GraphQL queries. Pass the authorization token as a connection parameter when initiating the WebSocket handshake.

### Connection lifecycle

1. Open a WebSocket connection to the GraphQL endpoint
2. Send the `connection_init` message with your auth token
3. Send a `subscribe` message with your subscription query
4. Receive events as they occur
5. Send `complete` to unsubscribe, or close the connection

## Persisted Subscriptions

Persisted subscriptions deliver events reliably through the outbox pattern. Events survive server restarts and are guaranteed to be delivered. Use these for critical business events.

### Customer KYC Updated

Fires when a customer's KYC verification status changes (e.g., from `PENDING_VERIFICATION` to `VERIFIED` or `REJECTED`).

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `kycVerification` | `KycVerification!` | New verification status: `PENDING_VERIFICATION`, `VERIFIED`, or `REJECTED` |
| `customer` | `Customer!` | The full customer object with updated data |

```graphql
subscription CustomerKycUpdated($customerId: UUID!) {
  customerKycUpdated(customerId: $customerId) {
    kycVerification
    customer {
      customerId
      email
      level
    }
  }
}
```

### Pending Credit Facility Collateralization Updated

Fires when the collateralization level of a pending credit facility changes due to price movements or collateral deposits.

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `state` | `PendingCreditFacilityCollateralizationState!` | `FULLY_COLLATERALIZED` or `UNDER_COLLATERALIZED` |
| `collateral` | `Satoshis!` | Current collateral amount in satoshis |
| `price` | `UsdCents!` | BTC/USD price at the time of the update |
| `recordedAt` | `Timestamp!` | When the event was recorded |
| `effective` | `Date!` | Effective date of the collateralization change |
| `pendingCreditFacility` | `PendingCreditFacility!` | The full pending credit facility object |

```graphql
subscription PendingFacilityCollateral($id: UUID!) {
  pendingCreditFacilityCollateralizationUpdated(pendingCreditFacilityId: $id) {
    state
    collateral
    price
    recordedAt
    effective
    pendingCreditFacility {
      pendingCreditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Pending Credit Facility Completed

Fires when a pending credit facility transitions to a terminal state (approved and activated, or denied).

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | `PendingCreditFacilityStatus!` | `PENDING_COLLATERALIZATION` or `COMPLETED` |
| `recordedAt` | `Timestamp!` | When the completion was recorded |
| `pendingCreditFacility` | `PendingCreditFacility!` | The full pending credit facility object |

```graphql
subscription PendingFacilityCompleted($id: UUID!) {
  pendingCreditFacilityCompleted(pendingCreditFacilityId: $id) {
    status
    recordedAt
    pendingCreditFacility {
      pendingCreditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Credit Facility Proposal Concluded

Fires when an approval process for a credit facility proposal reaches a final decision (approved or denied).

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | `CreditFacilityProposalStatus!` | Final status: `APPROVED`, `DENIED`, `CUSTOMER_DENIED`, etc. |
| `creditFacilityProposal` | `CreditFacilityProposal!` | The full proposal object |

```graphql
subscription ProposalConcluded($proposalId: UUID!) {
  creditFacilityProposalConcluded(creditFacilityProposalId: $proposalId) {
    status
    creditFacilityProposal {
      creditFacilityProposalId
      facilityAmount
      status
    }
  }
}
```

### Credit Facility Collateralization Updated

Fires when the collateralization level of an active credit facility changes due to price movements, collateral changes, or outstanding balance changes.

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `state` | `CollateralizationState!` | `FULLY_COLLATERALIZED`, `UNDER_MARGIN_CALL_THRESHOLD`, `UNDER_LIQUIDATION_THRESHOLD`, `NO_COLLATERAL`, or `NO_EXPOSURE` |
| `collateral` | `Satoshis!` | Current collateral amount in satoshis |
| `outstandingInterest` | `UsdCents!` | Outstanding accrued interest |
| `outstandingDisbursal` | `UsdCents!` | Outstanding disbursed principal |
| `recordedAt` | `Timestamp!` | When the event was recorded |
| `effective` | `Date!` | Effective date of the change |
| `price` | `UsdCents!` | BTC/USD price at the time of the update |
| `creditFacility` | `CreditFacility!` | The full credit facility object |

```graphql
subscription FacilityCollateral($facilityId: UUID!) {
  creditFacilityCollateralizationUpdated(creditFacilityId: $facilityId) {
    state
    collateral
    outstandingInterest
    outstandingDisbursal
    price
    recordedAt
    effective
    creditFacility {
      creditFacilityId
      status
      facilityAmount
    }
  }
}
```

## Ephemeral Subscriptions

Ephemeral subscriptions deliver transient events only while a client is actively subscribed. Events that occur while disconnected are not replayed. Use these for UI updates and non-critical notifications.

### Ledger Account CSV Export Uploaded

Fires when a CSV export of ledger account transactions finishes uploading and is ready for download.

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `documentId` | `UUID!` | ID of the generated CSV document, used to generate a download link |

```graphql
subscription CsvExportReady($accountId: UUID!) {
  ledgerAccountCsvExportUploaded(ledgerAccountId: $accountId) {
    documentId
  }
}
```

### Realtime Price Updated

Fires whenever the BTC/USD exchange rate changes. No arguments required.

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `usdCentsPerBtc` | `UsdCents!` | Current price of 1 BTC in USD cents |

```graphql
subscription PriceUpdates {
  realtimePriceUpdated {
    usdCentsPerBtc
  }
}
```

### Report Run Updated

Fires when a report run is created or its state changes (e.g., from `QUEUED` to `RUNNING` to `SUCCESS` or `FAILED`). No arguments required — delivers updates for all report runs.

**Payload fields:**

| Field | Type | Description |
|-------|------|-------------|
| `reportRunId` | `UUID!` | ID of the report run that was updated |

```graphql
subscription ReportUpdates {
  reportRunUpdated {
    reportRunId
  }
}
```

## Best Practices

- **Reconnection handling**: WebSocket connections can drop. Implement automatic reconnection with exponential backoff in your client.
- **Idempotent processing**: Persisted subscriptions may redeliver events in edge cases. Design your handlers to safely process the same event more than once.
- **Use persisted subscriptions for critical flows**: Customer KYC changes, credit facility state transitions, and collateralization updates are delivered reliably. Rely on these for business-critical integrations.
- **Use ephemeral subscriptions for UI**: Price updates and CSV export notifications are best suited for real-time UI feedback, not for durable processing.
- **Subscribe to specific entities**: Most subscriptions accept an entity ID to filter events. Subscribe only to the entities you need rather than processing all events client-side.
