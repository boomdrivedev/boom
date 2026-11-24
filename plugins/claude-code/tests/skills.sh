#!/usr/bin/env bash
# The plugin's skills, each at its path (claude-code-plugin, "The plugin
# teaches an agent to use Boom"). The script lists the files of skills/
# and reads none of them: no task reads the words of a skill
# (no-task-reads-markdown). The pinned Claude Code's strict validation in
# claude-code:typecheck reads them. Each case that fails prints
# `claude-code: skills: <what>`, and every case runs.

set -u
here=$(cd "$(dirname "$0")" && pwd)
plugin=$(cd "$here/.." && pwd)

failed=0
for file in \
  using-boom/SKILL.md \
  proposing-claims/SKILL.md \
  proposing-claims/kinds.md \
  writing-narratives/SKILL.md; do
  if [ ! -f "$plugin/skills/$file" ]; then
    echo "claude-code: skills: no skills/$file" >&2
    failed=1
  fi
done

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "claude-code: skills: each skill is at its path"
