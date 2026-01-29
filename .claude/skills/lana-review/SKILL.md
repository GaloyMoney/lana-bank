---
name: lana-review
description: Review code changes against LANA Bank's DDD patterns, event sourcing conventions, and architectural principles. Use when reviewing PRs, commits, or code changes.
---

# LANA Code Review

Review code for DDD principles and architectural patterns that require human judgment.

## Review Scope

$ARGUMENTS

If no specific files provided, review the current branch. Run these commands to gather context:
- `git branch --show-current` - get current branch name
- `gh pr view --json number,title,url,baseRefName` - get PR info (if PR exists)
- `git log --oneline main..HEAD | head -10` - get commits on this branch

To get the full diff:
- If PR exists: `gh pr diff`
- Otherwise: `git diff main..HEAD`

Exclude files listed in "Files to Ignore" from review.

## Review Context

- **Backwards compatibility: NOT a concern.** System is not deployed yet - no existing data or clients. Breaking changes are fine.

## Files to Ignore

`**/schema.graphql`, `.sqlx/`, `Cargo.lock`, `pnpm-lock.yaml`, `**/generated/**`, `**/*.snap`

## Review Checklist

### 1. Logic Placement (Functional Core, Imperative Shell)

**Core (entities):** Pure business logic and decisions
**Shell (use cases, adapters):** I/O, persistence, orchestration - no business rules

Note: We don't enforce strict immutability - Rust's ownership already prevents shared mutable state issues.

**"Tell, Don't Ask"** - entity methods encapsulate decisions:
```rust
// BAD: asking state, deciding outside
if entity.status() == Status::Active && entity.balance() > 0 {
    entity.do_something();
}

// GOOD: telling entity, it decides internally
entity.maybe_do_something()?;
```

**"Train Wrecks"** - method chaining that hides complexity:
```rust
// BAD: data access chain - reaching through object graph
let amount = facility.collateral().wallet().balance().amount();

// BAD: operation chain - hiding logic in sequential calls
let result = payment.validate().apply_fees().convert_currency().finalize();
```
Train wrecks violate encapsulation or hide business logic that should be explicit in a single well-named method.

**Watch for:**
- [ ] Complex conditionals in use cases → push to entity
- [ ] Business rules in GraphQL resolvers or HTTP handlers
- [ ] Use cases doing more than: load → call entity method → persist
- [ ] Method chains reaching through object graphs

### 2. Entities & Event Sourcing

**Entity rules:**
- [ ] Mutations are idempotent, return `Idempotent<>` wrapper
- [ ] Queries (`&self`) never fail - cannot return `Result`
- [ ] State modified only through events, never directly
- [ ] Each aggregate is its own consistency boundary - avoid modifying multiple aggregates in one transaction

**Private events** (in `entity.rs`):
- [ ] Small deltas tracking what changed
- [ ] Named in past tense

**Public events** (in `public/` subfolder):
- [ ] Snapshot-like: include current values, not just deltas
- [ ] Self-contained: consumers don't need history to understand state

### 3. Value Objects

Typically live in `primitives.rs`, but may live elsewhere if there's good reason.

- [ ] Immutable - no `&mut self` methods, create new instances instead
- [ ] Equality by value, not identity
- [ ] Encapsulate validation (e.g., `Amount` can't be negative)
- [ ] No business logic beyond self-validation

### 4. Module Boundaries

**Cross-module communication:**
- [ ] **Reads/queries:** Direct calls to lower-level modules are fine
- [ ] **State changes:** Use public events - other modules react via jobs
- [ ] One-way dependencies: higher-level (credit, deposit) → lower-level (customer, price, governance)

**General:**
- [ ] No domain knowledge in `lib/` utilities

### 5. Ubiquitous Language & Structure

- [ ] Names match business domain terms (not technical jargon)
- [ ] Same terms used in code, events, and business discussions
- [ ] Directory structure reveals domain story, not technical implementation
- [ ] APIs and concepts named by business meaning, not implementation details

```
// BAD: directories by technical role
/filters/, /handlers/, /validators/

// GOOD: directories by domain
/credit/, /customer/, /deposit/

// BAD: API named by implementation
SimpleConfig vs ComplexConfig

// GOOD: API named by business meaning
InternalConfig vs ExposedConfig
```

### 6. Emergent Design (YAGNI, KISS)

- [ ] Simplest solution that works - no premature optimization
- [ ] No features "we might need later" - build for current requirements
- [ ] No unnecessary abstractions - wait until patterns emerge
- [ ] Complexity should be justified by actual, not hypothetical, needs

### 7. Error Handling

- [ ] Errors are meaningful domain errors with business-relevant variants
- [ ] Use `From` implementations in `error.rs` to enable `?` propagation - avoid explicit `.map_err()` everywhere

### 8. Security & Audit

- [ ] No sensitive data in logs, traces or system triggered emails (credentials, PII)
- [ ] Audit logging for sensitive operations
- [ ] Use `*_without_audit` methods for internal/system operations - prefer this over passing "system" as a subject

## Output Format

```
**[SEVERITY]** file_path:line_number
Issue and why it violates DDD/architecture principles.
Suggested fix.
```

Severities: **CRITICAL** (event sourcing, data integrity) | **ERROR** (wrong layer, DDD violation) | **WARNING** (design concerns) | **INFO** (suggestions)

End with: Summary + Assessment (PASS / PASS WITH WARNINGS / NEEDS CHANGES)
