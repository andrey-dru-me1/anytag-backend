---
name: git-commit-message
description: Formulate a git commit message following the project's commit convention
modeSlugs:
  - code
  - debug
  - architect
  - ask
---
<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# git-commit-message Skill

Formulate a **git commit message** that conforms to the project's conventions documented in [`docs/GIT_WORKFLOW.md`](../../../docs/GIT_WORKFLOW.md) and [`AGENTS.md`](../../../AGENTS.md).

## When to Use

This skill applies whenever the agent or user needs to write a commit message for this repository, whether generating a new one or reviewing an existing one.

## Commit Format

```
<type>(<ticket-id>): <subject>

<body>

<footer>
```

The ticket ID (e.g., `ANY-1234`) **must** be placed in parentheses after the type.

### Allowed Types

| Type       | Usage                        |
| ---------- | ---------------------------- |
| `feat`     | New feature                  |
| `fix`      | Bug fix                      |
| `docs`     | Documentation changes        |
| `style`    | Code style (formatting, etc) |
| `refactor` | Code refactoring             |
| `test`     | Adding or updating tests     |
| `chore`    | Maintenance, deps, build     |

### Subject Rules

- Use **imperative mood**: "Add" not "Adds" or "Added"
- Start with a capital letter
- No period at the end
- Keep under **50 characters**

### Body (Optional)

- Explain **what** and **why**, not how
- Wrap at **72 characters**
- Use bullet points for multiple changes

### Footer (Optional)

- Reference issues: `Closes #123`, `Fixes #456`
- YouTrack reference: `See ANY-1234`
- Breaking changes: `BREAKING CHANGE: <description>`

## Examples

```
feat(ANY-1234): Add user authentication middleware
fix(ANY-5678): Resolve database connection timeout
docs(ANY-9012): Update API endpoint documentation
refactor(ANY-3456): Simplify error handling in handlers
chore(ANY-7890): Update Rust toolchain to 1.84.0
```

## Step-by-Step

1. **Determine the type** based on the nature of the changes (see table above).
2. **Identify the YouTrack ticket ID** from the branch name (`feature/ANY-1234-*`, `bugfix/ANY-5678-*`, etc.) or ask the user.
   - If no ticket ID is available and the change is trivial (chore, docs), use `chore` or `docs` without a ticket ID.
3. **Write the subject line** in imperative mood, capitalized, no period, ≤ 50 chars.
4. **Add a body** (optional but recommended for non-trivial changes):
   - Explain the motivation and reasoning.
   - Wrap at 72 characters.
5. **Add a footer** if relevant (breaking changes, issue references).
6. **Validate the final message** against the rules above.

## Important Constraint

This skill **formulates and displays** the commit message only. Do **not** run `git commit` — the user will commit themselves. Inspecting the working tree with `git diff`, `git status`, `git branch`, etc. is allowed.

## Key Project Rules (from AGENTS.md)

- **Every commit MUST reference a YouTrack ticket**: `type(TICKET-ID): subject`
- **Branch naming**: `feature/ANY-1234-description`, `bugfix/ANY-5678-description`, etc.
- **Merge strategy**: Merge commits (not squash, not rebase). See [`docs/GIT_WORKFLOW.md`](../../../docs/GIT_WORKFLOW.md).
- See [`AGENTS.md`](../../../AGENTS.md) for full project conventions.
