#!/bin/bash

# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors
# SPDX-License-Identifier: AGPL-3.0-only

#
# setup_agent_skills.sh
# =====================
# Creates a symlink from <agent-dir>/skills -> ../.agents/skills so that an AI
# coding assistant (Roo, Cursor, Copilot, Claude, etc.) picks up skill
# definitions from the version-controlled source.
#
# The agent's config directory (e.g. .roo/, .cursor/, .claude/) is typically
# gitignored, while .agents/ is tracked in version control. This script
# ensures the symlink exists after a fresh clone.
#
# Usage:
#   bash .agents/scripts/setup_agent_skills.sh <agent-dir>
#
# Examples:
#   bash .agents/scripts/setup_agent_skills.sh .roo      # Roo / Roo Code
#   bash .agents/scripts/setup_agent_skills.sh .cursor   # Cursor
#   bash .agents/scripts/setup_agent_skills.sh .claude   # Claude Code
#
# Idempotent — safe to run multiple times.

set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <agent-dir>"
    echo ""
    echo "Examples:"
    echo "  $0 .roo       # Roo / Roo Code"
    echo "  $0 .cursor    # Cursor"
    echo "  $0 .claude    # Claude Code"
    exit 1
fi

AGENT_DIR="$1"
AGENTS_SKILLS=".agents/skills"
SYMLINK_PATH="${AGENT_DIR}/skills"
RELATIVE_TARGET="../${AGENTS_SKILLS}"

# Ensure agent directory exists
if [ ! -d "$AGENT_DIR" ]; then
    echo "Creating ${AGENT_DIR}/"
    mkdir -p "$AGENT_DIR"
fi

# Remove existing symlink or directory if present
if [ -L "$SYMLINK_PATH" ] || [ -e "$SYMLINK_PATH" ]; then
    echo "Removing existing ${SYMLINK_PATH}"
    rm -rf "$SYMLINK_PATH"
fi

# Create the symlink
ln -s "$RELATIVE_TARGET" "$SYMLINK_PATH"
echo "Created symlink: ${SYMLINK_PATH} -> ${RELATIVE_TARGET}"

# Verify
if [ -L "$SYMLINK_PATH" ] && [ -d "$SYMLINK_PATH" ]; then
    echo "✓ Symlink is valid and points to an existing directory."
else
    echo "✗ Symlink is broken or missing. Ensure ${AGENTS_SKILLS} exists."
    exit 1
fi
