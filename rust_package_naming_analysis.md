# Rust Package Naming Analysis

## Overview
Analysis of mismatches between folder structure and Rust package names in the lana-bank monorepo.

## Current Naming Patterns

### Core Directory (`core/`)
Most packages follow the pattern `core-{folder-name}`, but some exceptions exist:

| Folder Name | Package Name | Status | Expected Name |
|-------------|--------------|--------|---------------|
| `access` | `core-access` | ✅ Consistent | - |
| `accounting` | `core-accounting` | ✅ Consistent | - |
| `credit` | `core-credit` | ✅ Consistent | - |
| `custody` | `core-custody` | ✅ Consistent | - |
| `customer` | `core-customer` | ✅ Consistent | - |
| `deposit` | `core-deposit` | ✅ Consistent | - |
| `money` | `core-money` | ✅ Consistent | - |
| `price` | `core-price` | ✅ Consistent | - |
| `document-storage` | `document-storage` | ⚠️ **Missing prefix** | `core-document-storage` |
| `governance` | `governance` | ⚠️ **Missing prefix** | `core-governance` |
| `public-id` | `public-id` | ⚠️ **Missing prefix** | `core-public-id` |

### Lana Directory (`lana/`)
Mixed pattern - some packages have `lana-` prefix, others don't:

| Folder Name | Package Name | Status | Pattern |
|-------------|--------------|--------|---------|
| `admin-server` | `admin-server` | ✅ No prefix pattern | - |
| `app` | `lana-app` | ✅ Prefixed pattern | - |
| `cli` | `lana-cli` | ✅ Prefixed pattern | - |
| `customer-server` | `customer-server` | ✅ No prefix pattern | - |
| `customer-sync` | `customer-sync` | ✅ No prefix pattern | - |
| `dashboard` | `dashboard` | ✅ No prefix pattern | - |
| `entity-rollups` | `entity-rollups` | ✅ No prefix pattern | - |
| `events` | `lana-events` | ✅ Prefixed pattern | - |
| `ids` | `lana-ids` | ✅ Prefixed pattern | - |
| `notification` | `notification` | ✅ No prefix pattern | - |
| `rbac-types` | `rbac-types` | ✅ No prefix pattern | - |
| `sim-bootstrap` | `sim-bootstrap` | ✅ No prefix pattern | - |
| `user-onboarding` | `user-onboarding` | ✅ No prefix pattern | - |

### Lib Directory (`lib/`)
All packages match their folder names exactly:

| Folder Name | Package Name | Status |
|-------------|--------------|--------|
| `audit` | `audit` | ✅ Consistent |
| `authz` | `authz` | ✅ Consistent |
| `bitgo` | `bitgo` | ✅ Consistent |
| `cloud-storage` | `cloud-storage` | ✅ Consistent |
| `job` | `job` | ✅ Consistent |
| `jwks-utils` | `jwks-utils` | ✅ Consistent |
| `komainu` | `komainu` | ✅ Consistent |
| `kratos-admin` | `kratos-admin` | ✅ Consistent |
| `outbox` | `outbox` | ✅ Consistent |
| `tracing-utils` | `tracing-utils` | ✅ Consistent |

## Issues Identified

### 1. Inconsistent Core Package Naming
**Problem**: Three packages in the `core/` directory are missing the `core-` prefix:
- `core/document-storage` → `document-storage` (should be `core-document-storage`)
- `core/governance` → `governance` (should be `core-governance`)  
- `core/public-id` → `public-id` (should be `core-public-id`)

**Impact**: 
- Breaks naming consistency within the core domain
- Makes it unclear these packages belong to the core business logic
- Could cause confusion in dependency management

### 2. Mixed Lana Package Naming
**Observation**: The `lana/` directory has two different naming patterns:
- **Prefixed pattern**: `lana-app`, `lana-cli`, `lana-events`, `lana-ids`
- **No prefix pattern**: `admin-server`, `customer-server`, `customer-sync`, `dashboard`, etc.

**Impact**:
- Inconsistent naming makes it unclear which packages are part of the main lana application
- No clear pattern for when to use prefixes vs. not

## Recommendations

### Option 1: Standardize Core Packages (Recommended)
Update the three inconsistent core packages to follow the established `core-` prefix pattern:
```toml
# core/document-storage/Cargo.toml
name = "core-document-storage"

# core/governance/Cargo.toml  
name = "core-governance"

# core/public-id/Cargo.toml
name = "core-public-id"
```

### Option 2: Establish Clear Lana Naming Rules
Define when to use `lana-` prefix in the lana directory:
- **Option 2a**: Prefix all packages: `lana-admin-server`, `lana-customer-server`, etc.
- **Option 2b**: Only prefix shared/library packages, leave service packages unprefixed
- **Option 2c**: Keep current mixed approach but document the pattern

### Option 3: Consider Workspace-level Naming
Consider using Cargo workspace features to manage package naming more systematically.

## Next Steps
1. **Immediate**: Fix the three core package naming inconsistencies
2. **Short-term**: Decide on and document the lana package naming strategy
3. **Long-term**: Consider automated checks to prevent future naming inconsistencies