---
id: intro
title: Lana Banking Platform
slug: /
sidebar_position: 1
---

# Lana Banking Platform

Lana is a modern banking core platform built for digital lending and custody operations. It provides comprehensive APIs and tools for managing credit facilities, customer accounts, and financial operations.

## Choose Your Path

<div className="row">
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>Technical Documentation</h3>
      </div>
      <div className="card__body">
        <p>Business processes, domain concepts, and admin panel procedures for bank staff.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="technical-documentation/">Technical Docs</a>
      </div>
    </div>
  </div>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>For External Developers</h3>
      </div>
      <div className="card__body">
        <p>Integrate with Lana's GraphQL APIs from external applications.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-external-developers/">External Dev Guide</a>
      </div>
    </div>
  </div>
</div>

<div className="row" style={{marginTop: '1rem'}}>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>For Internal Developers</h3>
      </div>
      <div className="card__body">
        <p>Local setup, frontend apps, domain architecture, and code patterns.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-internal-developers/">Internal Dev Guide</a>
      </div>
    </div>
  </div>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>For Platform Engineers</h3>
      </div>
      <div className="card__body">
        <p>System architecture, deployment, CI/CD, and data pipelines.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-platform-engineers/">Platform Guide</a>
      </div>
    </div>
  </div>
</div>

## Platform Highlights

| Capability | Description |
|------------|-------------|
| **Credit Facilities** | Full lifecycle management for loans and credit lines |
| **Multi-Currency** | Native support for USD and BTC |
| **Event Sourcing** | Complete audit trail of all operations |
| **GraphQL APIs** | Admin API (internal) + Customer API (external) |
| **Double-Entry Accounting** | Powered by Cala ledger |
| **Hexagonal Architecture** | Clean separation of concerns |

## Quick Links

### APIs
- [Admin API Reference](apis/admin-api/) - Full administrative operations
- [Customer API Reference](apis/customer-api/) - Customer-facing operations
- [Domain Events](apis/events/) - Event catalog

### Operations
- [Credit Management](technical-documentation/credit/) - Facility lifecycle
- [Accounting](technical-documentation/accounting/) - Financial operations

### Technical
- [System Architecture](for-platform-engineers/functional-architecture) - Comprehensive technical design
- [Data Models](for-platform-engineers/erds/) - Entity relationship diagrams
- [Local Development](for-internal-developers/local-development) - Dev environment setup

## Getting Started

New to Lana? Start with the [Getting Started](getting-started/) guide.
