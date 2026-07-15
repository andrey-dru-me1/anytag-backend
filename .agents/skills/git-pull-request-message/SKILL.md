---
name: git-pull-request-message
description: Formulate a pull request title and description following the project's PR conventions
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

# git-pull-request-message Skill

Formulate a **pull request title and description** that conforms to the project's conventions documented in [`docs/GIT_WORKFLOW.md`](../../../docs/GIT_WORKFLOW.md) and [`AGENTS.md`](../../../AGENTS.md).

## When to Use

This skill applies whenever the agent or user needs to create or review a pull request for this repository, including opening a new PR, updating an existing one, or suggesting improvements to a PR description.

## PR Title Format

```
[Type] TICKET-ID: Brief description of changes
```

The type is **PascalCase** (capital first letter only), followed by the YouTrack ticket ID in parentheses or after a colon/space (see examples below).

### Allowed Types

| Type       | Usage                        |
| ---------- | ---------------------------- |
| `Feature`  | New feature                  |
| `Bugfix`   | Bug fix                      |
| `Hotfix`   | Critical production fix      |
| `Refactor` | Code refactoring             |
| `Docs`     | Documentation changes        |
| `Chore`    | Maintenance, deps, build     |
| `Release`  | Release preparation          |

### Title Rules

- Use **PascalCase** for the type: `Feature`, `Bugfix`, `Refactor`, etc.
- Always include the **YouTrack ticket ID** (e.g., `ANY-1234`).
- Follow with a colon and a **brief description** in sentence case.
- Keep the entire title under **72 characters**.
- No period at the end.

### Title Examples

```
[Feature] ANY-1234: Add user authentication
[Bugfix] AT-5678: Resolve memory leak in database pool
[Refactor] AT-3456: Simplify error handling middleware
[Docs] ANY-9012: Update API endpoint documentation
[Chore] ANY-7890: Update Rust toolchain to 1.84.0
[Hotfix] ANY-3456: Patch CSRF vulnerability in auth endpoint
[Release] v1.2.0
```

> Release and small chore PRs may omit the ticket ID.

## PR Description

Use the following template. Include every section; omit sections only if explicitly not applicable.

```markdown
## Description

Brief summary of the changes and motivation.

## Changes Made

- Change 1
- Change 2
- Change 3

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Related Issues

Closes ANY-1234
Fixes ANY-5678

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated if needed
- [ ] No breaking changes introduced
```

### Section Guidelines

| Section        | Required | Content                                                                 |
| -------------- | -------- | ----------------------------------------------------------------------- |
| `Description`  | Yes      | What does this PR do? Why is it needed? Keep it concise (2-4 sentences) |
| `Changes Made` | Yes      | Bullet list of concrete changes. Group logically.                       |
| `Testing`      | Yes      | Checklist covering test coverage and verification steps.                |
| `Related Issues` | No    | YouTrack tickets or GitHub issues referenced. Use `Closes` / `Fixes`.   |
| `Checklist`    | Yes      | Standard quality checklist. Add extra items if the PR requires them.    |

## Step-by-Step

1. **Identify the PR type** based on the nature of the changes (see table above).
2. **Extract the YouTrack ticket ID** from the branch name (`feature/ANY-1234-*`, `bugfix/ANY-5678-*`, etc.) or from commit messages.
   - If no ticket ID exists and the change is trivial (chore, release), the ticket ID may be omitted.
3. **Write the title** in `[Type] TICKET-ID: Brief description` format:
   - PascalCase type
   - Ticket ID after the type
   - Concise description starting with a capital letter, no period
4. **Write the description body** using the template above:
   - **Description**: 2-4 sentences explaining motivation and approach.
   - **Changes Made**: List concrete changes, grouped logically.
   - **Testing**: Check off relevant testing items. Add custom steps if needed.
   - **Related Issues**: Reference tickets that this PR closes or fixes.
   - **Checklist**: Keep standard items, add extras if relevant.
5. **Validate the final PR** against the rules above.

## Key Project Rules (from AGENTS.md & GIT_WORKFLOW.md)

- **PR target**: `develop` branch (unless it's a hotfix targeting `master`).
- **Branch naming**: `feature/ANY-1234-description`, `bugfix/ANY-5678-description`, etc.
- **Merge strategy**: Merge commits (not squash, not rebase). See [`docs/GIT_WORKFLOW.md`](../../../docs/GIT_WORKFLOW.md).
- **Every PR MUST reference a YouTrack ticket ID** in the title (unless release or trivial chore).
- **Every commit within the PR MUST also reference a YouTrack ticket**: `type(TICKET-ID): subject`.
- All the items in **Testing** and **Checklist** must be unchecked. User will check them manually directly on GitHub.
- See [`AGENTS.md`](../../../AGENTS.md) for full project conventions.

## Related Skills

- [`git-commit-message`](../git-commit-message/SKILL.md) — Formulate individual commit messages within the PR.
