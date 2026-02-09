---
name: lana-agent-self-improve
description: Post-session retrospective that reflects on agent instruction files used during the session and makes targeted improvements. Run manually after completing real work.
---

# Agent Instruction Self-Improvement

Post-session retrospective tool. Run this after a work session to improve the instruction files you used. Do not run during active work.

$ARGUMENTS

## Step 1: Identify Files Consulted

Recall which instruction and context files you read or were influenced by during this session. These may include `CLAUDE.md`, skill files (`.claude/skills/`), command files (`.claude/commands/`), cursor rules (`.cursor/rules/`), documentation files (`AUTHN.md`, `AUTHZ.md`, `README.md`), or any other files that shaped your behavior.

List them explicitly before proceeding.

## Step 2: Evaluate Each File

For each file, ask:

- Was anything **wrong or outdated**?
- Was anything **missing** that would have materially helped this session's work?
- Was anything **confusing or misleading**?
- Was anything **unnecessarily verbose** or duplicated elsewhere?
- Could any section be **condensed** without losing meaning?

Skip files where you have no actionable feedback.

## Step 3: Make Targeted Edits

Edit files directly. Every edit must be justified by a concrete experience from this session — not hypothetical future benefit.

## Conservative Editing Philosophy

This is the most important constraint. These files are loaded into agent context on every session. Every character has a recurring cost.

- **Agents are smart readers.** Write principles, not step-by-step instructions. A principle covering 10 cases beats 3 detailed examples.
- **Additions must earn their place.** Only add what would have materially helped this session. "Might be useful someday" is not sufficient.
- **Removals are first-class improvements.** Deleting outdated, redundant, or verbose content is as valuable as adding new content.
- **Conciseness is a feature.** A paragraph rewritten as a sentence is a win.
- **No duplication.** If information exists in one file, don't repeat it in another.

## What NOT to Do

- Don't add information "just in case" — only what was actually needed
- Don't expand examples when a principle suffices
- Don't reorganize or reformat files for aesthetics
- Don't add meta-commentary about the improvement process
- Don't duplicate information that exists elsewhere in the repo

## Output

After making edits, briefly summarize what was changed and why, tied to specific session experiences.
