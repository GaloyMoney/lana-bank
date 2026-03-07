---
name: lana-docs
description: Update docs-site English content based on PR changes — API descriptions, event docs, feature pages, and sidebar config. Use this skill whenever a PR touches GraphQL schemas, domain events, core modules, or docs-site files. Also use it when asked to "update docs", "document this change", or "add documentation" for any code change. Spanish translations are handled automatically by the lingo.dev GitHub Action — never translate manually.
---

# LANA Docs Updater

Update the documentation site (`docs-site/`) to reflect changes made in the current PR. Only English content is maintained manually — Spanish translations are generated automatically by the lingo.dev GitHub Action after merge.

## Gather Context

$ARGUMENTS

If no specific scope is provided, analyze the current PR:

```bash
git branch --show-current
gh pr view --json number,title,url,baseRefName 2>/dev/null || echo "No PR yet"
git log --oneline main..HEAD
git diff main..HEAD --stat
```

## Off-Limits

**NEVER modify files under these paths** — they are frozen snapshots of released versions:
- `docs-site/versioned_docs/`
- `docs-site/versioned_sidebars/`
- `docs-site/schemas/versions/`

**NEVER create or modify Spanish translation files** — lingo.dev handles all i18n:
- `docs-site/i18n/es/`
- `docs-site/scripts/api-descriptions.es.json`
- `docs-site/scripts/event-descriptions.es.json`

Only modify files under `docs-site/docs/` (current/Next English content), `docs-site/scripts/` (English description files), and `docs-site/sidebars.js`.

## Step 1: Identify What Changed

Analyze the PR diff and categorize:

**A. API changes** — look for modifications in:
- `lana/admin-server/src/graphql/` (admin API schema/resolvers)
- `lana/customer-server/src/graphql/` (customer API schema/resolvers)

**B. Domain event changes** — look for modifications in:
- `core/*/src/**/public/` (public event definitions)

**C. Domain/feature changes** — look for modifications in:
- `core/` (domain logic)
- `lana/` (application layer)
- `lib/` (shared libraries)

**D. Direct docs changes** — already present in:
- `docs-site/docs/`
- `docs-site/scripts/`

## Step 2: Update English Documentation

### API changes

1. Add or update operation descriptions in `docs-site/scripts/api-descriptions.en.json`
2. For new operations, write a clear English description. For removed operations, keep stale entries — versioned docs may still reference them.

### Domain event changes

1. Add or update event descriptions in `docs-site/scripts/event-descriptions.en.json`
2. Regenerate event docs:
   ```bash
   cd docs-site && pnpm run generate-events-docs
   ```
   This produces both English and Spanish output (Spanish uses the existing `event-descriptions.es.json` which lingo.dev maintains).

### Domain/feature changes

- Update the relevant pages under `docs-site/docs/` that describe the changed functionality
- If a new concept or workflow was introduced, create a new doc page
- **Write at the right level of abstraction.** Document domain behavior changes (e.g., "obligations now transition via end-of-day batch processing" instead of "per-obligation scheduled jobs"). Skip implementation edge cases (e.g., validation rules for rare error paths) and details that are already obvious from the UI. If a user wouldn't need to know it to understand the system, leave it out.

### New doc pages

Check whether the new page needs a sidebar entry in `docs-site/sidebars.js`. Pages inside directories with `_category_.yml` are auto-discovered; top-level or non-standard placements need manual registration.

## Step 3: Validate

Run API description validation if any API-related files changed:

```bash
cd docs-site && pnpm run validate-api-descriptions
```

This checks that all current operations have descriptions and none use placeholder text. It validates both English and Spanish files — if Spanish descriptions are missing for new operations, that's expected and will be resolved by the lingo.dev translation workflow after merge.

## Step 4: Verify Build

```bash
cd docs-site && pnpm run build
```

This catches broken links, missing sidebar references, and MDX syntax errors. Fix any errors before finishing.

## Checklist

- [ ] Only files under `docs-site/docs/`, `docs-site/scripts/*.json` (English), and `docs-site/sidebars.js` were modified
- [ ] No versioned content touched (`versioned_docs/`, `versioned_sidebars/`, `schemas/versions/`)
- [ ] No Spanish/i18n files touched (`i18n/es/`, `*.es.json`)
- [ ] API description validation passes (if API changed)
- [ ] Event docs regenerated (if events changed)
- [ ] New pages registered in `sidebars.js` (if needed)
- [ ] `pnpm run build` passes with no broken links
