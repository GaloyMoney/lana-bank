---
name: lana-staging-verify
description: Verify that a deployed commit on staging works correctly. Analyzes the commit to understand what changed, runs feature-specific GraphQL tests against the staging admin API, and checks Honeycomb traces for healthy spans. Use after lana-deploy-monitor confirms a successful staging deployment.
---

# Verify Deployment on Staging

After a commit has been deployed to staging (typically confirmed by `lana-deploy-monitor`), verify that the changes work correctly by analyzing the commit, testing the staging API, and checking observability.

## Staging Environment

| Service | URL |
|---------|-----|
| Admin Panel | https://admin.staging.lana.galoy.io |
| Admin GraphQL | https://admin.staging.lana.galoy.io/graphql |

## Authentication

The superuser account for staging is `galoysuperuser@mailinator.com` (no password).

Staging Keycloak is at `https://auth.staging.lana.galoy.io`, realm `internal`, client `admin-panel`. The client uses PKCE (no password grant), but the account has no password so the login form auto-completes on email submission.

To get a token programmatically (authorization code flow with PKCE):

```python
import secrets, hashlib, base64, urllib.parse, re, html as html_mod, json, subprocess, tempfile

# 1. Generate PKCE
code_verifier = secrets.token_urlsafe(64)
code_challenge = base64.urlsafe_b64encode(
    hashlib.sha256(code_verifier.encode()).digest()
).rstrip(b'=').decode()

# Use a unique cookie file per auth attempt to avoid stale sessions
cookie_file = tempfile.mktemp(suffix='.txt', prefix='kc-cookies-')

# 2. Get login page (captures session cookies + form action)
auth_params = urllib.parse.urlencode({
    'client_id': 'admin-panel',
    'redirect_uri': 'https://admin.staging.lana.galoy.io/',
    'response_type': 'code',
    'scope': 'openid profile email',
    'code_challenge': code_challenge,
    'code_challenge_method': 'S256',
})
auth_url = f'https://auth.staging.lana.galoy.io/realms/internal/protocol/openid-connect/auth?{auth_params}'
r1 = subprocess.run(['curl', '-s', '-c', cookie_file, auth_url], capture_output=True, text=True)
form_action = html_mod.unescape(re.findall(r'action="([^"]+)"', r1.stdout)[0])

# 3. Submit email (no password) — Keycloak redirects with auth code
r2 = subprocess.run([
    'curl', '-s', '-b', cookie_file, '-X', 'POST', form_action,
    '-H', 'Content-Type: application/x-www-form-urlencoded',
    '-d', 'username=galoysuperuser@mailinator.com',
    '-o', '/dev/null', '-w', '%{redirect_url}'
], capture_output=True, text=True)
auth_code = re.search(r'code=([^&]+)', r2.stdout).group(1)

# 4. Exchange code for token
token_data = urllib.parse.urlencode({
    'client_id': 'admin-panel',
    'grant_type': 'authorization_code',
    'code': auth_code,
    'redirect_uri': 'https://admin.staging.lana.galoy.io/',
    'code_verifier': code_verifier,
})
r3 = subprocess.run([
    'curl', '-s', '-X', 'POST',
    'https://auth.staging.lana.galoy.io/realms/internal/protocol/openid-connect/token',
    '-H', 'Content-Type: application/x-www-form-urlencoded',
    '-d', token_data
], capture_output=True, text=True)
access_token = json.loads(r3.stdout)['access_token']
print(access_token)
```

Use the token in all GraphQL requests:
```
Authorization: Bearer {access_token}
```

**404 or 503 errors** typically mean staging is being reset. Wait 15 minutes for the rollout to complete, then retry.

## Workflow

### Step 1: Identify the commit

Resolve the target commit:

1. **`$ARGUMENTS` contains a commit SHA** — use it directly.
2. **`$ARGUMENTS` is empty** — use the latest commit on `origin/main` (`git fetch origin main && git log origin/main -1 --format='%H'`).

Display the commit SHA and message to the operator.

### Step 2: Analyze the commit

Understand what changed to determine what needs verification:

1. **Get changed files**: `git diff-tree --no-commit-id --name-only -r <sha>`
2. **Get the commit message**: `git log <sha> -1 --format='%B'`
3. **If it's a merge commit (PR)**, extract the PR number from the commit message and get the PR body:
   ```
   gh pr view <number> --json title,body,labels
   ```
4. **Read the actual code diff** for key files: `git show <sha> -- <relevant files>`

Classify the changes into categories:
- **GraphQL schema changes** — new queries/mutations/fields
- **Entity/domain logic changes** — new events, state transitions, business rules
- **API behavior changes** — new endpoints, modified responses
- **Infrastructure/config changes** — migrations, configuration
- **Frontend-only changes** — UI changes (limited backend verification possible)
- **Bug fixes** — specific scenario that was broken and should now work

### Step 3: Formulate a test plan

Based on the analysis, decide what to verify. Think about:

- **What GraphQL operations are affected?** Match changed code to available admin GraphQL queries in `bats/admin-gql/`. These `.gql` files are a catalog of available operations.
- **What OTEL spans should appear?** Look at `#[instrument]` annotations or `tracing::info_span!` calls in the changed code.
- **What behavior should be observable?** New data visible in queries, changed response shapes, new error handling.

**If you are unsure what to test — STOP and ask the operator.** Present:
- A summary of what you found in the commit
- Your best guess at what should be tested
- Specific questions about expected behavior

Do NOT proceed with testing if you don't have a clear plan.

### Step 4: Run feature-specific tests

Based on the test plan from Step 3, run targeted GraphQL queries against staging.

**Using BATS `.gql` files as templates:**
Read the relevant `.gql` file from `bats/admin-gql/` to get the exact query syntax, then use the same pattern as the BATS helpers (`bats/helpers.bash`) to execute it:

```bash
# Load and escape query (same as gql_admin_query in helpers.bash)
QUERY=$(cat bats/admin-gql/audit-logs.gql | tr '\n' ' ' | sed 's/"/\\"/g')

# Build variables with jq
VARIABLES=$(jq -n --argjson first 20 '{first: $first}')

# Build payload with jq (same as graphql_payload in helpers.bash)
PAYLOAD=$(jq -n --arg query "$QUERY" --argjson variables "$VARIABLES" \
  '{query: $query, variables: $variables}')

# Execute
curl -s -X POST https://admin.staging.lana.galoy.io/graphql \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD"
```

The key is using `jq -n --arg query` to safely embed the query string — this handles all JSON escaping.

**Important considerations:**
- Staging is a test environment — feel free to run both queries and mutations to fully exercise the changed code paths.
- Use mutations to create test entities, trigger workflows, and verify end-to-end behavior (e.g., create a customer, open a credit facility, record a deposit).
- Clean up is not required — staging data is ephemeral and regularly reset.

### Step 5: Report results

Provide a clear summary:

```
Staging Verification Report for <short-sha> (<commit message>)

## Changes Analyzed
- <brief description of what changed>

## Feature Verification
- <test 1>: PASS/FAIL — <details>
- <test 2>: PASS/FAIL — <details>

## Overall: PASS / FAIL / PARTIAL
<any notes or concerns>
```

## Guidelines

- **Staging is for breaking things.** Run mutations freely — create entities, trigger workflows, test error paths. No need to ask before mutating.
- **When in doubt, ask.** If you can't determine what to test from the commit, present your analysis and ask the operator.
- **Don't test what can't be tested.** Some changes (internal refactors, performance improvements) may not have observable API-level effects. Say so in the report.
- **Use the `.gql` catalog.** The `bats/admin-gql/` directory contains ~70 GraphQL operations — use them as a reference for available queries. Always read the `.gql` file first to get the exact query shape (field names, argument positions) rather than guessing.
- **Token caching.** Keycloak tokens typically last 5 minutes. If running many queries, re-authenticate if you get 401 responses.
