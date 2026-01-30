---
name: lana-review
description: Review code changes against LANA Bank's DDD patterns, event sourcing conventions, and architectural principles. Use when reviewing PRs, commits, or code changes.
---

# LANA Code Review

Review code for Hexagonal principles and architectural patterns that require human judgment.

## Review Scope

$ARGUMENTS

If no specific files provided, review the current branch. Run these commands to gather context:
- `git branch --show-current` - get current branch name
- `gh pr view --json number,title,url,baseRefName` - get PR info (if PR exists)
- `git log --oneline main..HEAD` - get commits on this branch

To get the full diff:
- If PR exists: `gh pr diff`
- Otherwise: `git diff main..HEAD`

Exclude files listed in "Files to Ignore" from review.

## Context Management

**Use subagents for exploration tasks.** When answering review questions that require deep codebase exploration, delegate to a subagent (Task tool with `subagent_type=Explore`) rather than reading many files directly. This keeps the main review context focused on the actual changes.

Examples of when to use subagents:
- "Does this repo method follow existing patterns?" → subagent explores other repo implementations
- "Is this event structure consistent with es-entity conventions?" → subagent examines es-entity usage across the codebase
- "How do other modules handle this pattern?" → subagent surveys similar code in other modules
- "What's the design intent of this library?" → subagent reads library docs and example usages

The subagent returns a concise answer without polluting your context with dozens of file reads. Only read files directly when you need the exact content for the review itself (e.g., the files being changed).

## Review Context

- **Backwards compatibility: NOT a concern.** System is not deployed yet - no existing data or clients. Breaking changes are fine.

## Files to Ignore

`.sqlx/`, `Cargo.lock`, `pnpm-lock.yaml`, `**/generated/**`, `**/*.snap`

## Review Checklist

### 1. Hexagonal

The basic code flow in the backend follows the following structure:
- Adapter layer (can be GQL resolver, or a Job implementation run() fn)
=> Where the execution begins -> calls the application layer (heuristic is 1 use case function is called from the adapter layer - never more)
- An application layer use case generally has the structure:
=> authorization / audit check (can be omitted if its an internal only use case) - load entity - mutate entity - persist entity
=> if the use case is complex and needs to execute multiple things delegating to additional internal 'domain services' is advised. They have the same fractal structure - just with the authorization check omitted.

The pattern separates code within the application into two roles: a functional core that contains all business logic and decisions, and an imperative shell that handles all I/O and side effects.

**Core (entities in `entity.rs`):** Pure business logic and decisions. No I/O, no database calls, no external services. Given the same inputs, always produces the same outputs. Almost all conditional (if / else statements) should be here and unit tested.

**Shell (in `mod.rs` use cases):** Orchestrates the core. Handles I/O, persistence, external calls. Contains no business rules - just loads data, calls entity methods, and persists results. There should be almost no conditionals in this code. Only if the use case truely branches but not to facilitate entity updates.

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
- [ ] Mutations are idempotent, return `Idempotent<>` wrapper or use `Result<Idempotent, _>` when pre-condition violation needs to be signaled
- [ ] Queries (`&self`) never fail - cannot return `Result`
- [ ] State modified only through events, never directly
- [ ] Each aggregate is its own consistency boundary - avoid modifying multiple aggregates in one transaction
- [ ] Entity mutations should have unit tests

**Private events** (in `entity.rs`):
- [ ] Small deltas tracking what changed
- [ ] Named in past tense

**Public events** (in `public/` subfolder):
- [ ] Snapshot-like: include current values, not just deltas
- [ ] Self-contained: consumers don't need history to understand state
- [ ] Should be integration tested to document which use-case triggers which public event

### 3. Value Objects

Typically live in `primitives.rs`, but may live elsewhere if there's good reason.

- [ ] Immutable - no `&mut self` methods, create new instances instead
- [ ] Equality by value, not identity
- [ ] Encapsulate validation (e.g., `Amount` can't be negative)
- [ ] No business logic beyond self-validation
- [ ] Good place for entities to delegate business logic too as they are simpler to test (no need for state setup via event history as in entities)
- [ ] Good for wrapping underlying primitive types (like `UsdCents(Decimal)`) to give domain specific meaning

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

// BAD: API named by implementation detail
CachedPriceService vs DirectPriceService

// GOOD: API named by business meaning
RealtimePrice vs SettlementPrice
```

### 6. Emergent Design (YAGNI, KISS)

- [ ] Simplest solution that works - no premature optimization
- [ ] No features "we might need later" - build for current requirements
- [ ] No unnecessary abstractions - wait until patterns emerge
- [ ] Complexity should be justified by actual, not hypothetical, needs

### 7. Error Handling

- [ ] Domain errors are meaningful with business-relevant variants (e.g., `InsufficientCollateral`, `FacilityNotActive`)
- [ ] Infrastructure errors (`SqlxError`, `FileError`) can wrap lower-level errors - they don't need business meaning
- [ ] Use `From` implementations in `error.rs` to enable `?` propagation - avoid explicit `.map_err()` everywhere

### 8. Security & Audit

- [ ] No sensitive data in logs, traces or system triggered emails (credentials, PII)
- [ ] Audit logging for sensitive operations (mutations should be logged)

## Severity Philosophy

**Be conservative with severity.** Higher severity items (WARNING and above) should only be raised when there is:
1. A clear violation of design intent (not just a stylistic preference)
2. Real, demonstrable harm (not theoretical concerns)
3. A fix that doesn't increase overall complexity

**Pragmatic rule-breaking is often correct.** We sometimes deviate from patterns to keep things simple. If a "violation" keeps complexity low with no real downside, don't flag it as a problem. The goal is working, maintainable software - not pattern purity.

### Issue Severity Levels

All severity levels are for **issues/concerns only** - things that may need attention or action.

| Severity | Use When | Examples |
|----------|----------|----------|
| **CRITICAL** | Data integrity at risk, events malformed, security vulnerability | Mutable event fields, missing audit on sensitive ops, SQL injection |
| **ERROR** | Clear architectural violation causing real harm | Business logic in resolver with no entity method, cross-module state mutation without events |
| **WARNING** | Design concern that should be addressed, fix is straightforward | Complex conditional in use case that belongs in entity, train wreck hiding important logic |
| **SUGGESTION** | Minor improvement idea, optional, no real harm if ignored | "Could extract this to a value object", "Consider renaming for clarity", pattern deviation that's acceptable but worth noting |

**Do NOT elevate to WARNING or above:**
- Style preferences (naming, formatting beyond conventions)
- "Cleaner" alternatives that add abstraction
- Pattern suggestions for code that works fine as-is
- Theoretical future concerns ("this might cause issues if...")

### Highlights (Positive Observations)

Separately from issues, note **good patterns** worth highlighting. These demonstrate you understood the code and reinforce good practices. Use these sparingly - only for genuinely notable good decisions, not routine correct code.

Examples of what to highlight:
- Good "Tell, Don't Ask" encapsulation that could easily have been done wrong
- Clean module boundary decisions
- Well-designed value objects or error types
- Thoughtful test coverage for edge cases

## Output Format

Structure the review with clear sections:

```
## Summary
Brief description of what the PR does.

## Highlights
Notable good patterns or decisions (optional - only if there are genuinely good things worth calling out).

- `file_path:line_number` - Brief description of what's good and why.

## Issues

**[SEVERITY]** `file_path:line_number`
Issue and why it matters (not just "violates pattern X").
Suggested fix (only if severity is WARNING or above).

## Assessment
PASS / PASS WITH WARNINGS / NEEDS CHANGES
```

**Assessment criteria:**
- **PASS**: No issues at WARNING or above
- **PASS WITH WARNINGS**: Has WARNING items but no ERROR/CRITICAL
- **NEEDS CHANGES**: Has ERROR or CRITICAL items
