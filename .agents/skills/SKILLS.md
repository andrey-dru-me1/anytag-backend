<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Skills Overview

This directory contains skill definitions that provide the AI with specialized instructions for recurring tasks. Each skill has a `SKILL.md` file describing its purpose, trigger conditions, and step-by-step procedures.

The AI evaluates the user's request against all available skills before responding. If a skill matches, its instructions are loaded and followed precisely.

---

## Available Skills

### [`git-commit-message`](./git-commit-message/SKILL.md)

Formulate a git commit message following the project's commit convention.

- **When triggered**: User or agent needs to write a commit message for this repository.
- **Format**: `<type>(<ticket-id>): <subject>` with mandatory YouTrack ticket reference.
- **See**: [`docs/GIT_WORKFLOW.md`](../docs/GIT_WORKFLOW.md)

### [`git-pull-request-message`](./git-pull-request-message/SKILL.md)

Formulate a pull request title and description following the project's PR conventions.

- **When triggered**: User or agent needs to create or review a GitHub pull request for this repository.
- **Format**: `[Type] TICKET-ID: Brief description` with mandatory YouTrack ticket reference in the title.
- **Includes**: Full PR description template covering changes, testing, and checklist.
- **See**: [`docs/GIT_WORKFLOW.md`](../docs/GIT_WORKFLOW.md)
- **Related**: [`git-commit-message`](./git-commit-message/SKILL.md)

### [`reuse-compliance`](./reuse-compliance/SKILL.md)

Add SPDX license/copyright headers to new files to maintain REUSE compliance.

- **When triggered**: Creating a new source file or fixing a missing SPDX header.
- **Enforcement**: CI-checked but not enforced by `cargo check` — manual compliance required.
- **See**: [`docs/REUSE.md`](../docs/REUSE.md)

---

## Adding a New Skill

To add a new skill:

1. Create a new subdirectory under `.agents/skills/<skill-name>/`.
2. Create a `SKILL.md` file with the following frontmatter:

   ```yaml
   ---
   name: <skill-name>
   description: <one-line description of when this skill applies>
   modeSlugs:
     - <mode-slug> # e.g., code, architect, debug
   ---
   ```

3. Document the instructions clearly, with examples.
4. Add an entry to this file's **Available Skills** section describing when the skill should be used.

---

## Setup

AI coding assistants (Roo, Cursor, Copilot, Claude Code, etc.) read skills from their own config directory (e.g. `.roo/skills/`, `.cursor/skills/`). These directories are typically gitignored. The canonical source is `.agents/skills/` (version-controlled). A symlink bridges them.

### Automatic (recommended)

Run the setup script after cloning, passing your AI tool's config directory:

```bash
bash .agents/scripts/setup_agent_skills.sh .roo      # Roo / Roo Code
bash .agents/scripts/setup_agent_skills.sh .cursor   # Cursor
bash .agents/scripts/setup_agent_skills.sh .claude   # Claude Code
```

The script is idempotent — safe to run multiple times.

### Manual

```bash
ln -s ../.agents/skills <agent-dir>/skills
```

For example:

```bash
ln -s ../.agents/skills .roo/skills
```

### Verification

Check that the symlink exists and is valid:

```bash
ls -la <agent-dir>/skills
# Expected: lrwxr-xr-x ... <agent-dir>/skills -> ../.agents/skills
```
