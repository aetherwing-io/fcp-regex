#!/bin/bash
# Wrapper for fcp-regex — provides install instructions if binary is missing.
if command -v fcp-regex &>/dev/null; then
  exec fcp-regex "$@"
else
  echo "fcp-regex not found. Install:" >&2
  echo "  curl -fsSL https://aetherwing-io.github.io/fcp-regex/install.sh | sh" >&2
  echo "" >&2
  echo "Or build from source:" >&2
  echo "  cargo install --path ~/projects/fcp/fcp-regex" >&2
  echo "" >&2
  echo "Or run /fcp:setup in Claude Code to install all FCP dependencies." >&2
  exit 1
fi
