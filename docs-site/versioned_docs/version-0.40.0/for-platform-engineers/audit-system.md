---
id: audit-system
title: Audit System
sidebar_position: 11
---

# Audit and Record System

This document describes the audit logging system for compliance and security.

## Overview

The audit system provides:

- Complete operation history
- Compliance reporting
- Security monitoring
- Forensic analysis

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AUDIT SYSTEM                                 │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Application Layer                       │   │
│  │        (GraphQL Resolvers, Domain Services)              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Audit Service                          │   │
│  │           (Intercepts and logs operations)               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Audit Repository                       │   │
│  │              (Persistent storage)                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     PostgreSQL                           │   │
│  │                  (audit_entries table)                   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Audit Entry Structure

```rust
pub struct AuditEntry {
    pub id: AuditEntryId,
    pub timestamp: DateTime<Utc>,
    pub subject: SubjectId,
    pub subject_type: SubjectType,
    pub action: String,
    pub object: String,
    pub object_id: Option<String>,
    pub outcome: AuditOutcome,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub metadata: serde_json::Value,
}

pub enum SubjectType {
    User,
    System,
    ApiClient,
}

pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}
```

## Audit Categories

| Category | Description | Examples |
|----------|-------------|----------|
| Authentication | Login/logout events | User login, session expiry |
| Authorization | Permission decisions | Access denied, role check |
| Data Access | Read operations | View customer, export report |
| Data Modification | Write operations | Create facility, update terms |
| System Events | Background processes | Job execution, sync completed |

## Integration Points

### GraphQL Middleware

```rust
pub struct AuditMiddleware {
    audit_service: Arc<AuditService>,
}

impl Extension for AuditMiddleware {
    async fn execute(&self, ctx: &ExtensionContext<'_>, operation_name: Option<&str>, next: NextExecute<'_>) -> Response {
        let start = Instant::now();
        let subject = ctx.data::<SubjectId>().cloned();

        let response = next.run(ctx, operation_name).await;

        // Log the operation
        if let Some(subject) = subject {
            self.audit_service.log(AuditEntry {
                subject,
                action: operation_name.unwrap_or("unknown").to_string(),
                outcome: if response.is_ok() { AuditOutcome::Success } else { AuditOutcome::Failure },
                ..Default::default()
            }).await.ok();
        }

        response
    }
}
```

### Domain Service Auditing

```rust
impl CreditService {
    pub async fn create_facility(
        &self,
        subject: &SubjectId,
        input: CreateFacilityInput,
    ) -> Result<CreditFacility> {
        let facility = self.do_create_facility(input).await?;

        // Audit the operation
        self.audit.log(AuditEntry {
            subject: subject.clone(),
            action: "create_facility".to_string(),
            object: "credit_facility".to_string(),
            object_id: Some(facility.id.to_string()),
            outcome: AuditOutcome::Success,
            metadata: json!({
                "amount": facility.amount,
                "customer_id": facility.customer_id,
            }),
            ..Default::default()
        }).await?;

        Ok(facility)
    }
}
```

## Query API

### GraphQL Query

```graphql
query GetAuditLogs($filter: AuditFilter!, $first: Int, $after: String) {
  auditEntries(filter: $filter, first: $first, after: $after) {
    edges {
      node {
        id
        timestamp
        subject
        action
        object
        objectId
        outcome
        metadata
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

### Filter Options

```graphql
input AuditFilter {
  subjectId: ID
  action: String
  object: String
  outcome: AuditOutcome
  startDate: DateTime
  endDate: DateTime
}
```

## Retention Policy

| Data Type | Retention Period |
|-----------|------------------|
| Authentication logs | 2 years |
| Authorization logs | 2 years |
| Transaction logs | 7 years |
| System logs | 1 year |

## Compliance Reports

### User Activity Report

```graphql
query UserActivityReport($userId: ID!, $period: DateRange!) {
  userActivityReport(userId: $userId, period: $period) {
    totalActions
    actionsByType {
      action
      count
    }
    timeline {
      date
      actions
    }
  }
}
```

### Access Report

```graphql
query AccessReport($objectType: String!, $period: DateRange!) {
  accessReport(objectType: $objectType, period: $period) {
    totalAccesses
    uniqueUsers
    byAction {
      action
      count
    }
  }
}
```

## Security Monitoring

### Anomaly Detection

Monitor for unusual patterns:

- Multiple failed login attempts
- Access outside normal hours
- Bulk data exports
- Privilege escalation attempts

### Alerts

```rust
pub struct AuditAlertService {
    audit_repo: AuditRepo,
    notification_service: NotificationService,
}

impl AuditAlertService {
    pub async fn check_for_anomalies(&self) -> Result<()> {
        // Check for suspicious patterns
        let failed_logins = self.audit_repo
            .count_failed_logins_last_hour()
            .await?;

        if failed_logins > 10 {
            self.notification_service
                .send_security_alert("High number of failed logins detected")
                .await?;
        }

        Ok(())
    }
}
```

