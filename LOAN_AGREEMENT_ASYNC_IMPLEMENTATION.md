# Asynchronous Loan Agreement Generation Implementation

This document outlines the implementation of asynchronous loan agreement generation following the existing CSV export pattern in the codebase.

## Overview

The loan agreement generation has been converted from a synchronous process to an asynchronous job-based system that:

1. **Creates a loan agreement record** when requested
2. **Spawns an async job** to generate the PDF 
3. **Updates the record** when generation completes or fails
4. **Provides download links** for completed agreements

## Architecture

The implementation follows the DDD (Domain Driven Design) and hexagonal architecture patterns established in the codebase, similar to the accounting CSV export functionality.

### Core Components

#### 1. Domain Layer (`core/document-storage/src/loan_agreement/`)

- **`entity.rs`** - Contains the `LoanAgreement` entity and events
- **`primitives.rs`** - Basic types and enums (`LoanAgreementId`, `LoanAgreementStatus`)
- **`repo.rs`** - Repository for persistence 
- **`error.rs`** - Error types specific to loan agreements
- **`generate.rs`** - PDF generation logic with template rendering
- **`job.rs`** - Asynchronous job implementation
- **`templates/loan_agreement.hbs`** - Handlebars template for PDF generation

#### 2. Application Layer (`lana/app/src/document_storage/`)

- **`mod.rs`** - App-level wrapper that exposes loan agreements functionality

#### 3. GraphQL Layer (`lana/admin-server/src/graphql/`)

- **`loan_agreement.rs`** - GraphQL types and resolvers
- **`schema.graphql`** - Updated schema with new mutations

## Key Features

### Async Job Processing

Following the same pattern as CSV exports:

```rust
// Job spawned immediately when agreement is requested
self.jobs.create_and_spawn_in_op::<GenerateLoanAgreementConfig<Perms>>(
    &mut db,
    agreement.id,
    GenerateLoanAgreementConfig {
        loan_agreement_id: agreement.id,
        _phantom: std::marker::PhantomData,
    },
).await?;
```

### Status Tracking

Three statuses track the generation lifecycle:
- **`Pending`** - Initial state when agreement is created
- **`Completed`** - PDF generated and uploaded successfully  
- **`Failed`** - Generation or upload failed

### PDF Generation

- Uses **Handlebars** for HTML template rendering
- Mock PDF generation (ready for real PDF library integration)
- Uploads generated files to cloud storage
- Tracks storage path and filename

### Error Handling

Comprehensive error handling with:
- Template rendering errors
- Storage upload errors
- Generation failures
- Audit trail for all operations

## API Usage

### Generate Loan Agreement (Async)

```graphql
mutation LoanAgreementGenerate($input: LoanAgreementGenerateInput!) {
  loanAgreementGenerate(input: $input) {
    customerId
    storagePath
    filename
  }
}
```

**Input:**
```json
{
  "input": {
    "customerId": "uuid-here"
  }
}
```

**Response:**
```json
{
  "data": {
    "loanAgreementGenerate": {
      "customerId": "uuid-here", 
      "storagePath": null,        // Will be null until job completes
      "filename": null            // Will be null until job completes
    }
  }
}
```

### Generate Download Link

```graphql
mutation LoanAgreementDownloadLinkGenerate($input: LoanAgreementDownloadLinkGenerateInput!) {
  loanAgreementDownloadLinkGenerate(input: $input) {
    loanAgreementId
    link
  }
}
```

## Database Schema

### New Tables

```sql
-- Main loan agreements table
CREATE TABLE loan_agreements (
  id UUID PRIMARY KEY,
  customer_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Event sourcing events table  
CREATE TABLE loan_agreement_events (
  id UUID NOT NULL REFERENCES loan_agreements(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence)
);
```

## Event Sourcing

The implementation uses event sourcing with these events:

### `LoanAgreementEvent::Initialized`
```json
{
  "type": "initialized", 
  "id": "uuid",
  "customer_id": "uuid",
  "audit_info": {...}
}
```

### `LoanAgreementEvent::FileGenerated`  
```json
{
  "type": "file_generated",
  "storage_path": "loan_agreements/uuid.pdf", 
  "filename": "loan_agreement_uuid.pdf",
  "audit_info": {...}
}
```

### `LoanAgreementEvent::GenerationFailed`
```json
{
  "type": "generation_failed",
  "error": "Error message", 
  "audit_info": {...}
}
```

## Job Processing

### Job Configuration

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct GenerateLoanAgreementConfig<Perms> {
    pub loan_agreement_id: LoanAgreementId,
    pub _phantom: std::marker::PhantomData<Perms>,
}
```

### Job Execution Flow

1. **Retrieve agreement** from database
2. **Begin database transaction**
3. **Generate PDF** using template and customer data
4. **Upload to storage** (cloud storage)
5. **Update agreement** with success/failure
6. **Commit transaction**

## Template System

### Template Data Structure

The PDF template receives:
```json
{
  "customer_id": "uuid",
  "customer_name": "John Doe", 
  "loan_amount": "$10,000.00",
  "interest_rate": "5.5%",
  "term_months": 12,
  "generation_date": "2024-12-01"
}
```

### Template Location
`core/document-storage/src/loan_agreement/templates/loan_agreement.hbs`

## Integration Points

### Authorization

Permissions are enforced through the existing authorization system:

```rust
CoreDocumentStorageAction::LOAN_AGREEMENT_CREATE
CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE  
CoreDocumentStorageAction::LOAN_AGREEMENT_GENERATE_DOWNLOAD_LINK
```

### Audit Trail

All operations are logged through the audit system with proper context and subject tracking.

### Storage

Uses the existing cloud storage abstraction for file upload and download link generation.

## Testing

The BATS test expects:

```bash
# Test verifies async generation returns expected fields
loan_agreement_generate_result=$(graphql_output '.data.loanAgreementGenerate')
customer_id=$(echo $loan_agreement_generate_result | jq -r '.customerId')
storage_path=$(echo $loan_agreement_generate_result | jq -r '.storagePath') 
filename=$(echo $loan_agreement_generate_result | jq -r '.filename')
```

## Next Steps

### Production Readiness

1. **Real PDF Generation** - Replace mock implementation with actual PDF library
2. **Template Management** - Move templates to configurable location
3. **Customer Data Integration** - Fetch real customer and loan data
4. **Error Recovery** - Add retry logic for failed generations
5. **Monitoring** - Add metrics for job success/failure rates

### PDF Library Options

- **wkhtmltopdf** - Command line tool for HTML→PDF
- **Headless Chrome** - Use Chrome/Chromium for rendering
- **rust-pdf** libraries - Native Rust PDF generation

### Template Enhancement

- **Dynamic templates** based on loan type
- **Multi-language support**
- **Digital signature integration**
- **Customer-specific branding**

## Benefits of Async Implementation

1. **Better User Experience** - No waiting for PDF generation
2. **Scalability** - Jobs can be processed in background workers
3. **Reliability** - Failed generations don't block the API
4. **Auditability** - Full tracking of generation status
5. **Consistency** - Follows established patterns in the codebase

This implementation provides a solid foundation for production loan agreement generation while maintaining consistency with the existing codebase architecture and patterns.