---
name: lana-frontend-admin-panel
description: Implementation guide for building pages, components, forms, tables, and features in the admin-panel frontend. Use when writing or modifying code in apps/admin-panel/ or apps/shared-web/.
---

# Admin Panel Frontend Implementation Guide

Before writing code, explore existing patterns in the codebase. Look at similar pages/components for reference. Follow what exists — don't invent new patterns.

**Maintaining this guide:** When you introduce a new pattern, improve an existing convention, or change how something works in the admin-panel, update this skill file to reflect the change. This document should always match the actual codebase.

## Type Safety

- **No `any`** — use codegen types or define your own. If truly impractical, add `// TODO: type this - <reason>`
- **Exhaustive enum checks** — every `switch` on a GraphQL enum must handle all variants and use `const exhaustiveCheck: never = value` in `default`. No silent `default` that swallows new variants. See `customer-status-badge.tsx` for the pattern.
- **Derive types from codegen** — use `NonNullable<GetXxxQuery["field"]>` for component props. Never hand-write types that codegen generates.
- **No type assertions** (`as`) unless justified with a comment explaining why it's safe.

## Components & Architecture

- **Client components only** — always `"use client"`. No React Server Components for data fetching. The only exception is `app/layout.tsx` (root layout).
- **Use `@lana/web` components** — check `apps/shared-web/src/ui/` before creating any UI element. Never duplicate shadcn components or use raw HTML when a component exists. Import as `@lana/web/ui/<component>`.
- **Modular & composable** — extract reusable pieces into their own components. Split components into smaller files if they become too large. Each component should have a single responsibility.
- **DDD in frontend** — the `app/` directory mirrors backend bounded contexts (customers, credit-facilities, deposits, etc.). New features go in route folders matching the backend domain name.
- **File naming** — `kebab-case.tsx` for components, `use-kebab-case.ts` for hooks, `page.tsx`/`layout.tsx` for routes, `list.tsx` for list components, `create.tsx`/`action-name.tsx` for dialogs.
- **Page components are thin** — they wrap content in a `Card` and delegate to child components. See existing `page.tsx` files.
- **Detail layouts use tabs** — via `useTabNavigation` hook with URL persistence. See existing `[entity-id]/layout.tsx` files.
- **Breadcrumbs** — set via `useBreadcrumb()` context from detail layouts. See existing layouts.
- **Loading states** — use `DetailsPageSkeleton` for detail pages, `Skeleton` from `@lana/web/ui/skeleton` for custom states. Pattern: `if (loading && !data)` to allow stale cache data while refreshing.

## GraphQL & Data Layer

- **Define `gql` operations inline** at the top of the component file that uses them. Use generated hooks (`useXxxQuery`, `useXxxMutation`) — never call `useQuery`/`useMutation` directly.
- **Cache updates over refetchQueries** — prefer `update` callback with `cache.modify()` or `cache.writeQuery()` using the mutation response. Only use `refetchQueries` when the response doesn't contain enough data. See `users/create.tsx` or `disbursals/create.tsx` for patterns.
- **Pagination is Relay-style cursor-based** — use `PaginatedTable` from `components/paginated-table/` with `fetchMore`. Cache policies use `relayStylePagination()` with `keyArgs` for sort/filter. See `lib/apollo-client/client.tsx` for cache config.
- **Fragments** — use named fragments for shared field selections across queries.
- **Conditional fetching** — use `skip` option to defer queries until needed (e.g., `skip: !isDialogOpen`).
- **After schema changes** — run `make sdl` then `pnpm codegen` in admin-panel. Never edit `lib/graphql/generated/`.

## Forms & Dialogs

- **Dialog state managed by parent** — parent passes `open`/`setOpen` props. Dialog component handles form state internally.
- **Reset on close** — always reset form state, validation errors, and Apollo mutation state (`reset()`) when dialog closes. Implement a `handleClose` function.
- **Validation** — client-side validation before submission. Display both validation errors and mutation errors together in `text-destructive`.
- **Toast notifications** — use `toast` from `sonner` for success/error feedback after mutations.
- **Button loading state** — disable submit during loading, use loading prop in Button component.

## Tables

- **`PaginatedTable`** — for server-paginated GraphQL data with sort, filter, and cursor-based pagination. Located in `components/paginated-table/`.
- **`DataTable`** — for static/local datasets without server pagination. Located in `components/data-table/` or `@lana/web/components/data-table`.
- **Column definitions** — use `Column<T>` type with `key`, `label` (translated), optional `sortable`, `filterValues`, `render` function. See existing list components.
- **Navigation** — use `navigateTo` prop for row click navigation (usually to detail page via `publicId`).

## i18n

- **Every user-facing string through `useTranslations()`** — never hardcode display text in JSX.
- **Only edit `messages/en.json`** — Spanish (`es.json`) is auto-generated by lingo.dev GitHub Action. Never edit it manually.
- **Namespace by feature** — keys follow `Feature.SubFeature.key` pattern. Group by: `title`, `description`, `columns`, `labels`, `placeholders`, `actions`, `messages`, `errors`.
- **Multiple namespaces** — when a component needs translations from different features, use multiple `useTranslations()` calls with different namespaces.

## Styling & Theme

- **Theme colors only** — use Tailwind classes referencing CSS variables (`text-primary`, `bg-destructive`, `border-muted`). Never raw hex/rgb/oklch values in component code.
- **New colors in shared-web** — add to both `:root` and `.dark` in `apps/shared-web/global.css`, then register in the `@theme inline` block. Colors use OKLCH color space.
- **`cn()` for conditional classes** — import from `@lana/web/utils`. Use for merging Tailwind classes conditionally.

## Shared Web (`@lana/web`)

- **UI components** (`@lana/web/ui/`): card, button, dialog, input, select, tabs, table, badge, skeleton, separator, breadcrumb, dropdown-menu, alert, alert-dialog, sheet, popover, tooltip, hover-card, command, scroll-area, accordion, collapsible, checkbox, radio-group, switch, toggle, toggle-group, calendar, textarea, progress, loading-spinner, avatar, pagination, label, field, form, input-group, input-otp, kbd, empty, button-group, menubar, sonner (toast).
- **Custom components** (`@lana/web/components/`): DataTable, DetailsCard/DetailsGroup/DetailItem, DateWithTooltip.
- **Hooks** (`@lana/web/hooks/`): `useIsMobile()`, `useBreakpointUp(bp)`, `useBreakpointDown(bp)`, `useMediaQuery(query)`.
- **Utilities** (`@lana/web/utils/`): `cn()`, `formatDate()`, `parseUTCDate()`, `getUTCYear()`, `formatUTCMonthName()`, `formatUTCMonthYear()`, `formatUTCDateOnly()`, `formatSpacedSentenceCaseFromSnakeCase()`.


## Cypress Testing

- **Language-independent** — use `data-testid` attributes for selectors, never translated text. Add `data-testid` to all interactive elements.
- **Extend existing specs** — add tests to existing `cypress/e2e/*.cy.ts` files for the same domain. New spec files only for new domains.
- **Shared database** — all tests share one DB. Use unique identifiers (timestamps, random suffixes) to avoid conflicts.
- **Wait for loading** — assert `[data-testid="loading-skeleton"]` doesn't exist before interacting.
- **Custom commands** — use `cy.graphqlRequest()`, `cy.createCustomer()`, `cy.takeScreenshot()`, `cy.KcLogin()` from `cypress/support/commands.ts`. Explore existing commands before writing new helpers.
- **Screenshots** — use `cy.takeScreenshot(name)` at key steps for visual documentation.

## Environment & Auth

- **Env vars** — use `env` from `@/env` (T3 Env + Zod validated). Never access `process.env` directly. Add new vars to `env.ts`.
- **Auth is automatic** — Keycloak PKCE flow + Apollo auth link handles JWT injection. Session timeout via `react-idle-timer`. Don't handle auth in feature code.

## Code Readability & React Best Practices

- **Readable JSX** — keep JSX shallow and scannable. Prefer named components over deeply nested ternaries or inline logic. If a block of JSX needs a comment to explain what it renders, extract it into a named component instead.
- **Descriptive naming** — components, hooks, handlers, and variables should have clear, self-documenting names. Prefer `handleSubmitEmail` over `handleClick`, `CustomerStatusBadge` over `StatusBadge`.
- **Functional components with hooks only** — no class components.
- **Single responsibility** — pages compose components, don't contain all logic. One component = one purpose.
- **Lift state only when needed** — keep state closest to where it's used.
- **Stable keys** — entity IDs for list keys, not array indices (static lists excepted).
- **No inline complex handlers in JSX** — extract to named functions or `useCallback` if more than a single setter.
- **Controlled components** — all form inputs are controlled via `useState` + `value`/`onChange`.
- **Error handling** — try/catch in mutation handlers, display errors in UI, `toast.error()` for user feedback. Never silently swallow errors.
- **Follow React idioms and best practices** — use hooks (`useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useContext`) correctly per React docs. When a pattern feels awkward or fights the framework, step back and look for the idiomatic approach. If deviating from React best practices is unavoidable, add a `// NOTE:` comment explaining why.

