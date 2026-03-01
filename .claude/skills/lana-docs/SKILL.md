---
name: lana-docs
description: Update docs-site content based on PR changes. Handles API description validation, Spanish translations, and ensures versioned docs are never modified.
---

# LANA Docs Updater

Update the documentation site to reflect changes made in the current PR.

## Gather Context

$ARGUMENTS

If no specific scope is provided, analyze the current PR. Run these commands to gather context:
- `git branch --show-current` - get current branch name
- `gh pr view --json number,title,url,baseRefName` - get PR info (if PR exists)
- `git log --oneline main..HEAD` - get commits on this branch
- `gh pr diff` or `git diff main..HEAD` - get the full diff

## Off-Limits: Versioned Content

**NEVER modify files under these paths:**
- `docs-site/versioned_docs/` - frozen documentation snapshots
- `docs-site/versioned_sidebars/` - frozen sidebar configs
- `docs-site/schemas/versions/` - frozen schema snapshots
- `docs-site/i18n/es/docusaurus-plugin-content-docs/version-*/` - frozen Spanish translations for released versions

These directories contain point-in-time snapshots of released versions. They must remain untouched. Only `current/` (i.e. "Next") content should be updated.

## Step 1: Identify What Changed

Analyze the PR diff and categorize changes:

### A. API Changes (GraphQL schema modifications)
Look for changes in:
- `lana/admin-server/src/graphql/` — admin API schema or resolvers
- `lana/customer-server/src/graphql/` — customer API schema or resolvers

### B. Domain/Feature Changes
Look for changes in:
- `core/` — domain logic modules
- `lana/` — application layer
- `lib/` — shared libraries

### C. Direct Docs Changes
Look for changes already made in:
- `docs-site/docs/` — English documentation (current version)
- `docs-site/scripts/` — docs tooling

## Step 2: Update Documentation Content

Based on what changed in the PR, update the relevant docs under `docs-site/docs/` (current/Next version only).

### For API changes:
1. If GraphQL schema changed, regenerate the schema file first: `make sdl`
2. Update API operation descriptions in **both** language files:
   - `docs-site/scripts/api-descriptions.json` (English)
   - `docs-site/scripts/api-descriptions.es.json` (Spanish)
3. For new operations, add descriptions. For removed operations, leave stale entries (they may be needed by versioned docs).

### For domain event changes:
If public events changed (files under `core/*/src/**/public/`):
1. Update description files in both languages:
   - `docs-site/scripts/event-descriptions.json` (English)
   - `docs-site/scripts/event-descriptions.es.json` (Spanish)
2. Regenerate the event docs:
   ```bash
   cd docs-site && pnpm run generate-events-docs && pnpm run generate-events-docs -- --locale es
   ```

### For domain/feature changes:
- Update the relevant pages under `docs-site/docs/` that describe the changed functionality
- Focus on `for-operators/`, `for-developers/`, and `for-platform-engineers/` sections as appropriate
- If a new concept or workflow was added, add or update the corresponding doc page

### For new doc pages:
When adding a new page under `docs-site/docs/`, check if it needs to be registered in `docs-site/sidebars.js`. Pages placed inside a directory with `_category_.yml` are auto-discovered, but top-level or non-standard placements may require a manual sidebar entry.

### For direct docs changes:
- These are already in place; proceed to translation (Step 4)

## Step 3: Validate API Descriptions

**Run this whenever API-related files were changed:**

```bash
cd docs-site && pnpm run validate-api-descriptions
```

This validates that:
- All current operations have descriptions in both English and Spanish
- No descriptions use default placeholder text

If validation fails, fix the missing or placeholder descriptions before proceeding.

## Step 4: Translate to Spanish

For **every** file changed or added under `docs-site/docs/` (current version), create or update the corresponding Spanish translation.

### Translation mapping:
| English source | Spanish target |
|---|---|
| `docs-site/docs/<path>` | `docs-site/i18n/es/docusaurus-plugin-content-docs/current/<path>` |

### Translation rules:
- Translate all prose content to Spanish
- Do **NOT** translate: code blocks, CLI commands, variable names, file paths, URLs, proper nouns (product names, library names), or technical identifiers
- Preserve all markdown formatting, frontmatter structure, and admonition syntax exactly
- Keep the same heading hierarchy and document structure
- If a Spanish translation already exists, update only the sections that correspond to the English changes — do not rewrite the entire file
- For `_category_.yml` files, translate only the `label` and `description` fields

### Quality:
- Use Latin American Spanish, consistent with existing translations
- Match the tone and terminology used in existing Spanish docs under `docs-site/i18n/es/`
- When unsure about a domain term, check existing translations for precedent before choosing a new translation

## Step 5: Verify Docs Build

After all changes are made, run a build to catch broken links, missing pages, and rendering issues:

```bash
cd docs-site && pnpm run build
```

This will surface:
- Broken internal links (e.g. renamed or removed pages)
- Missing sidebar references
- MDX syntax errors

Fix any build errors before finishing.

## Checklist

Before finishing, verify:

- [ ] No files under `versioned_docs/`, `versioned_sidebars/`, `schemas/versions/`, or `i18n/.../version-*/` were modified
- [ ] All changed/added English docs have corresponding Spanish translations in `i18n/es/.../current/`
- [ ] If API schemas changed: `pnpm run validate-api-descriptions` passes in `docs-site/`
- [ ] If event schemas changed: event description files updated in both languages and `generate-events-docs` run
- [ ] If new pages added: `sidebars.js` updated if needed
- [ ] `pnpm run build` passes in `docs-site/` (no broken links or rendering errors)
- [ ] Markdown formatting is preserved (frontmatter, admonitions, code blocks)
