#!/usr/bin/env bash
# The pinned Claude Code with a scratch home and no traffic beyond the
# command's, so a plugin's check reads none of the user's configuration or
# credentials and writes nothing of theirs; none of it needs the network
# (core.3). The plugin's project (claude-code:typecheck) and the
# marketplace's check (root:marketplace) run `claude` through it.
#
# timber's plugins/claude.sh at boomdrivedev/timber 1adfecc7, for boom:
# its scratch folder is named boom-claude, and its rule is boom's core.3.

set -u

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/boom-claude.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/home"
for k in $(env | grep -Eo '^(CLAUDE|ANTHROPIC_)[A-Z_]*=' | tr -d =); do
  unset "$k"
done
export HOME=$scratch/home CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1

claude "$@"
