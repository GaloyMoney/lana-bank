# URL Conversion Summary: Path-based to Subdomain-based Routing

This document summarizes the changes made to convert the application from path-based URLs to subdomain-based URLs.

## Conversion Overview

**Before:**
- Admin Panel: `http://localhost:4455/admin`
- Customer Portal: `http://localhost:4455/app`

**After:**
- Admin Panel: `http://admin.localhost:4455`
- Customer Portal: `http://app.localhost:4455`

## Files Modified

### 1. Documentation and Configuration Files

#### `README.md`
- Updated service table URLs from path-based to subdomain-based
- Updated setup instructions with new URLs
- Changed admin panel login instructions
- Updated customer portal access instructions

#### `Makefile`
- Updated health check URL from `localhost:4455/admin` to `admin.localhost:4455`

### 2. Development Configuration

#### `dev/Tiltfile`
- Updated playground link from `localhost:4455/admin/graphql` to `admin.localhost:4455/graphql`
- Updated admin panel link from `localhost:4455/admin` to `admin.localhost:4455`
- Updated customer portal link from `localhost:4455/app` to `app.localhost:4455`
- Changed admin panel environment variables:
  - `NEXT_PUBLIC_BASE_PATH`: `/admin` → `/`
  - `NEXT_PUBLIC_CORE_ADMIN_URL`: `/admin/graphql` → `/graphql`
- Changed customer portal environment variables:
  - `NEXT_PUBLIC_BASE_PATH`: `/app` → `/`
  - `NEXT_PUBLIC_CORE_APP_URL`: `/app/graphql` → `/graphql`
- Updated readiness probe ports from oathkeeper proxy to direct app ports

#### `dev/bin/start-cypress-stack.sh`
- Updated health check URL for admin panel
- Updated environment variables:
  - `NEXT_PUBLIC_BASE_PATH`: `/admin` → `/`
  - `NEXT_PUBLIC_CORE_ADMIN_URL`: `/admin/graphql` → `/graphql`

### 3. Authentication Configuration (Kratos)

#### `dev/ory/admin/kratos.yml`
- Updated base URL from `localhost:4455` to `admin.localhost:4455`
- Updated CORS allowed origins
- Updated selfservice URLs
- Changed login UI URL from `localhost:4455/admin/login` to `admin.localhost:4455/login`
- Changed error UI URL from `localhost:4455/admin/errored` to `admin.localhost:4455/errored`

#### `dev/ory/customer/kratos.yml`
- Updated base URL from `localhost:4455` to `app.localhost:4455`
- Updated CORS allowed origins
- Updated selfservice URLs
- Changed login UI URL from `localhost:4455/app/login` to `app.localhost:4455/login`
- Changed error UI URL from `localhost:4455/app/errored` to `app.localhost:4455/errored`

### 4. Testing Configuration

#### `bats/helpers.bash`
- Updated GraphQL endpoints:
  - `GQL_APP_ENDPOINT`: `${OATHKEEPER_PROXY}/app/graphql` → `http://app.localhost:4455/graphql`
  - `GQL_ADMIN_ENDPOINT`: `${OATHKEEPER_PROXY}/admin/graphql` → `http://admin.localhost:4455/graphql`
- Updated authentication flow URLs for both customer and admin login endpoints
- Updated self-service API URLs to use subdomain format

#### `apps/admin-panel/cypress.config.ts`
- Changed base URL from `localhost:4455/admin` to `admin.localhost:4455`

#### `apps/admin-panel/cypress/run.sh`
- Updated admin URL variable from `localhost:4455/admin` to `admin.localhost:4455`

#### `apps/admin-panel/cypress/support/commands.ts`
- Updated GraphQL URL from `localhost:4455/admin/graphql` to `admin.localhost:4455/graphql`

#### `apps/customer-portal/cypress.config.ts`
- Changed base URL from `localhost:4455` to `app.localhost:4455`

### 5. Application Configuration

#### `apps/admin-panel/README.md`
- Updated login instructions with new admin panel URL

#### `apps/admin-panel/app/auth/session.tsx`
- Simplified GraphQL URL configuration to always use `/graphql` (removed dev/prod path difference)

#### `apps/customer-portal/env.ts`
- Updated default URLs:
  - `NEXT_PUBLIC_CORE_URL`: `localhost:4455` → `app.localhost:4455`
  - `NEXT_PUBLIC_KRATOS_PUBLIC_API`: `localhost:4455` → `app.localhost:4455`

## Key Implementation Notes

1. **Oathkeeper Rules**: The existing oathkeeper rules in `dev/ory/oathkeeper_rules.yaml` already support subdomain routing through regex patterns (`<[a-zA-Z0-9-.:]+>`), so no changes were needed there.

2. **NextJS Configuration**: The NextJS apps already use environment variables for base paths (`NEXT_PUBLIC_BASE_PATH`), so they automatically adapt to the new subdomain structure when the environment variables are updated.

3. **Cookie Domain**: The Kratos cookie domain should be updated to `.localhost` to work across subdomains (this was not explicitly set in the original configuration but is handled by the domain changes).

4. **Port Mapping**: The Docker Compose configuration remains unchanged as it only exposes the oathkeeper port (4455) which handles the routing.

## Testing

After these changes, the following URLs should work:
- Admin Panel: `http://admin.localhost:4455`
- Customer Portal: `http://app.localhost:4455`
- Admin GraphQL Playground: `http://admin.localhost:4455/graphql`
- Customer GraphQL Endpoint: `http://app.localhost:4455/graphql`

## Benefits of This Change

1. **Consistency**: Local development now matches production subdomain structure
2. **Simplified Configuration**: No need for path-based routing configuration
3. **Better Isolation**: Each application has its own subdomain namespace
4. **Easier Testing**: Test URLs match production patterns more closely